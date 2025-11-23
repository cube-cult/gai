{
  description = "gai";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-parts,
      ...
    }@inputs:

    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem =
        {
          pkgs,
          system,
          lib,
          ...
        }:
        let
          rustToolchain = inputs.fenix.packages.${system}.stable.toolchain;

          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

          versionInfo = craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; };
          src = craneLib.cleanCargoSource ./.;

          commonArgs = {
            inherit (versionInfo) pname version;
            inherit src;

            nativeBuildInputs = [
              pkgs.pkg-config
              pkgs.libclang.lib
            ];

            buildInputs = [
              pkgs.openssl.dev
            ];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          inherit (pkgs.stdenv) isLinux;
        in
        {
          packages =
            let
              gai = craneLib.buildPackage (
                commonArgs
                // {
                  inherit cargoArtifacts src;
                  nativeBuildInputs =
                    commonArgs.nativeBuildInputs
                    ++ lib.optional isLinux [
                      pkgs.autoPatchelfHook
                    ];
                }
              );
            in
            {
              inherit gai;
              default = gai;
            };

          devShells.default =
            let
              inherit (pkgs)
                mkShell
                just
                openssl
                pkg-config
                ;
            in
            mkShell {
              name = "gai-shell";
              packages = [
                just
                rustToolchain
                pkg-config
              ];

              buildInputs = [
                openssl
              ];

              # This fixes issues with `cargo run` not being to find
              # OpenSSL libraries at runtime in certain cases during
              # development.
              #
              # Relevant thread: https://discourse.nixos.org/t/program-compiled-with-rust-cannot-find-libssl-so-3-at-runtime/27196
              LD_LIBRARY_PATH = lib.makeLibraryPath [ openssl ];
            };
        };
    };
}
