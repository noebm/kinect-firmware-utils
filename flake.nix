{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.11";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      naersk-lib = pkgs.callPackage naersk {};
    in {
      devShell = with pkgs;
        mkShell {
          buildInputs = [cargo rustc rustfmt pre-commit rustPackages.clippy];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };

      packages = rec {
        default = kinect-firmware-utils;
        kinect-firmware-utils = naersk-lib.buildPackage ./.;

        kinect-firmware-blob = with pkgs;
          stdenv.mkDerivation rec {
            name = "kinect-firmware-blob";

            src = fetchurl {
              url = "http://download.microsoft.com/download/F/9/9/F99791F2-D5BE-478A-B77A-830AD14950C3/KinectSDK-v1.0-beta2-x86.msi";
              hash = "sha256-gXdkWRz/esw9Z4xbxl3Icks9JDYRwQENqywY0N7dQiE=";
            };

            buildInputs = [p7zip];
            unpackPhase = ''
              7z e -y -r ${src} "UACFirmware.*" > /dev/null
            '';
            installPhase = ''
              FW_FILE=$(ls UACFirmware.* | cut -d ' ' -f 1)
              cat $FW_FILE > $out
            '';
          };

        kinect-udev-rules = with pkgs;
          stdenv.mkDerivation rec {
            name = "kinect-udev-rules";

            src = ./udev;
            RULES = "55-kinect_audio.rules";
            RULES_IN = "${RULES}.in";

            patchPhase = ''
              substitute ${RULES_IN} ${RULES} \
                --subst-var-by LOADER_PATH ${kinect-firmware-utils}/bin/kinect-firmware-utils \
                --subst-var-by FIRMWARE_PATH ${kinect-firmware-blob}
            '';

            installPhase = ''
              install -D ${RULES} $out/lib/udev/rules.d/${RULES}
            '';
          };
      };
      apps = rec {
        kinect-firmware-utils = utils.lib.mkApp {
          drv = self.packages.${system}.kinect-firmware-utils;
        };
        default = kinect-firmware-utils;
      };

      nixosModules.default = {system, ...}: {
        services.udev.packages = [self.packages."${system}".kinect-udev-rules];
      };
    });
}
