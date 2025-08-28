{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    wix-extract.url = "github:noebm/wix-extract";
    wix-extract.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
      wix-extract,
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        kinect-packages = import ./pkgs {
          inherit pkgs;
          crane-lib = crane.mkLib pkgs;
          wix-extract = wix-extract.apps.${system}.default.program;
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };

        packages = rec {
          inherit (kinect-packages) kinect-firmware-blob kinect-udev-rules kinect-firmware-utils;
          default = kinect-firmware-utils;
        };
        apps = rec {
          kinect-firmware-utils = flake-utils.lib.mkApp {
            drv = self.packages.${system}.kinect-firmware-utils;
          };
          firmware-status = flake-utils.lib.mkApp {
            name = "firmware-status";
            drv = self.packages.${system}.kinect-firmware-utils;
          };
          default = kinect-firmware-utils;
        };

        nixosModules.default =
          { system, ... }:
          {
            services.udev.packages = [ self.packages."${system}".kinect-udev-rules ];
          };
      }
    );
}
