{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    sussg.url = "github:cube-cult/sussg/main";
  };

  outputs = {
    self,
    nixpkgs,
    sussg,
  }: let
    system = "x86_64-linux";
  in {
    devShells."${system}".default = let
      pkgs = import nixpkgs {inherit system;};
    in
      pkgs.mkShell {
        packages = [
          sussg.packages.${system}.default
        ];
      };
  };
}
