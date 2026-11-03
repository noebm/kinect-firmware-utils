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
    }:
    let
      # See: https://github.com/numtide/flake-utils/issues/124
      # nixpkgs 26.11 no longer supports Intel Darwin.
      systems = builtins.filter (system: system != "x86_64-darwin") flake-utils.lib.defaultSystems;
      per-system = flake-utils.lib.eachSystem systems (
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
            inherit (kinect-packages) kinect-firmware;
            default = kinect-firmware;
          };
          apps = rec {
            kinect-audio-firmware-upload =
              (flake-utils.lib.mkApp {
                name = "kinect-audio-firmware-upload";
                drv = self.packages.${system}.kinect-firmware;
              })
              // {
                meta.description = "Upload Kinect audio firmware";
              };
            kinect-audio-firmware-status =
              (flake-utils.lib.mkApp {
                name = "kinect-audio-firmware-status";
                drv = self.packages.${system}.kinect-firmware;
              })
              // {
                meta.description = "Query Kinect audio firmware status";
              };
            kinect-tilt = (flake-utils.lib.mkApp {
              name = "kinect-tilt";
              drv = self.packages.${system}.kinect-firmware;
            }) // {
              meta.description = "Read or set Kinect tilt in degrees";
            };
            default = kinect-audio-firmware-upload;
          };

          checks = nixpkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            nixos-module-downmix = import ./nix/tests/module.nix {
              inherit nixpkgs self system;
            };
          };
        }
      );

    in
    per-system
    // {
      nixosModules.default = import ./nix/module.nix { inherit self; };
    };
}
