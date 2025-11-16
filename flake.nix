{
  description = "gai";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    rust-overlay.url = "github:oxalica/rust-overlay";

    sussg.url = "github:nuttycream/sussg";
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
        { pkgs, system, ... }:
        {
          _module.args = {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [
                (import inputs.rust-overlay)
              ];
            };
          };

          packages =
            let
              gai =
                let
                  inherit (pkgs)
                    rustPlatform
                    openssl
                    pkg-config
                    ;
                in
                rustPlatform.buildRustPackage {
                  name = "gai";
                  src = ./.;

                  buildInputs = [
                    openssl
                  ];

                  nativeBuildInputs = [
                    pkg-config
                  ];

                  cargoHash = "sha256-7reFi36k8a707QmtcsBqlQ712TBSKjFWGnBU0NE8/uw=";
                };
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
                rust-bin
                openssl
                pkg-config
                ;
              sussg = (inputs.sussg.packages.${system}.default);
            in
            mkShell {
              name = "gai-shell";
              packages = [
                just
                rust-bin.stable.latest.default
                sussg
              ];

              nativeBuildInputs = [
                openssl
                pkg-config
              ];
            };
        };
    };
}
