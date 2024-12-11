{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    wix-extract.url = "github:noebm/wix-extract";
    wix-extract.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    wix-extract,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      crane-lib = crane.lib.${system};
    in {
      devShells.default = with pkgs;
        mkShell {
          buildInputs = [cargo rustc rustfmt pre-commit rustPackages.clippy];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };

      packages = rec {
        default = kinect-firmware-utils;
        kinect-firmware-utils = crane-lib.buildPackage {
          src = crane-lib.cleanCargoSource (crane-lib.path ./.);
        };

        kinect-firmware-blob = kinect-firmware-blob_1_8;

        kinect-firmware-blob_1_8 = with pkgs;
          stdenv.mkDerivation {
            pname = "kinect-firmware-blob";
            version = "1.8";

            src = fetchurl {
              url = "https://download.microsoft.com/download/E/C/5/EC50686B-82F4-4DBF-A922-980183B214E6/KinectRuntime-v1.8-Setup.exe";
              hash = "sha256-9NQUP7DwqNJ2iJwHe/yK9Cv+mcEoytq14xa/AVqYWOk=";
            };
            buildInputs = [wix-extract.packages.${system}.default p7zip];

            unpackPhase = ''
              wix-extract $src -d $TMP
              7z e -y -r $TMP/KinectDrivers-v1.8-x86.WHQL.msi "UACFirmware" > /dev/null
            '';

            installPhase = ''
              cp UACFirmware $out
            '';
          };

        kinect-firmware-blob_1_0_beta2 = with pkgs;
          stdenv.mkDerivation rec {
            pname = "kinect-firmware-blob";
            version = "1.0-beta2";

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
        kinect-firmware-utils = flake-utils.lib.mkApp {
          drv = self.packages.${system}.kinect-firmware-utils;
        };
        default = kinect-firmware-utils;
      };

      nixosModules.default = {system, ...}: {
        services.udev.packages = [self.packages."${system}".kinect-udev-rules];
      };
    });
}
