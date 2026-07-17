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
      per-system = flake-utils.lib.eachDefaultSystem (
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
            kinect-firmware-utils = flake-utils.lib.mkApp {
              drv = self.packages.${system}.kinect-firmware;
            };
            firmware-status = flake-utils.lib.mkApp {
              name = "firmware-status";
              drv = self.packages.${system}.kinect-firmware;
            };
            default = kinect-firmware-utils;
          };

          checks = nixpkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
            nixos-module-downmix =
              let
                evaluatedConfig = nixpkgs.lib.nixosSystem {
                  inherit system;
                  modules = [
                    self.nixosModules.default
                    {
                      hardware.kinect-audio = {
                        enable = true;
                        downmix = {
                          enable = true;
                          gain = 0.4;
                          sourceNodeName = "test-kinect-array";
                          virtualSourceName = "test-kinect-mono";
                          virtualSourceDescription = "Test Kinect Mono";
                        };
                      };
                      system.stateVersion = "26.05";
                    }
                  ];
                };
                filterModule = builtins.head evaluatedConfig.config.services.pipewire.extraConfig.pipewire."91-kinect-audio-mono"."context.modules";
                graph = filterModule.args."filter.graph";
                mixer = builtins.head graph.nodes;
                pipewireDropin = pkgs.writeText "kinect-downmix.conf" (builtins.toJSON {
                  "context.modules" = [ filterModule ];
                });
                pipewireConfigDir = pkgs.runCommand "kinect-downmix-pipewire-config" { } ''
                  mkdir -p $out/pipewire.conf.d
                  cp ${pkgs.pipewire}/share/pipewire/pipewire.conf $out/pipewire.conf
                  ln -s ${pipewireDropin} $out/pipewire.conf.d/91-kinect-downmix.conf
                '';
              in
              assert filterModule.name == "libpipewire-module-filter-chain";
              assert filterModule.args ? "filter.graph";
              assert filterModule.args ? "capture.props";
              assert filterModule.args ? "playback.props";
              assert graph.inputs == [
                "mixer:In 1"
                "mixer:In 2"
                "mixer:In 3"
                "mixer:In 4"
              ];
              assert graph.outputs == [ "mixer:Out" ];
              assert mixer.control == {
                "Gain 1" = 0.4;
                "Gain 2" = 0.4;
                "Gain 3" = 0.4;
                "Gain 4" = 0.4;
              };
              assert filterModule.args."capture.props"."target.object" == "test-kinect-array";
              assert filterModule.args."capture.props"."audio.channels" == 4;
              assert filterModule.args."playback.props"."node.name" == "test-kinect-mono";
              assert filterModule.args."playback.props"."audio.channels" == 1;
              pkgs.runCommand "nixos-module-downmix-check"
                {
                  nativeBuildInputs = [ pkgs.pipewire ];
                }
                ''
                  export PIPEWIRE_CONFIG_DIR=${pipewireConfigDir}
                  export PIPEWIRE_RUNTIME_DIR="$TMPDIR/runtime"
                  export XDG_RUNTIME_DIR="$PIPEWIRE_RUNTIME_DIR"
                  mkdir -p "$PIPEWIRE_RUNTIME_DIR"

                  pw-config -n pipewire.conf -r merge context.modules > merged.conf

                  pipewire -c pipewire.conf > pipewire.log 2>&1 &
                  pipewire_pid=$!
                  sleep 1

                  if ! kill -0 "$pipewire_pid" 2>/dev/null; then
                    wait "$pipewire_pid" || true
                    cat pipewire.log
                    exit 1
                  fi

                  kill "$pipewire_pid"
                  wait "$pipewire_pid" || true
                  touch $out
                '';
          };
        }
      );

    in
    per-system
    // {
      nixosModules.default =
        {
          pkgs,
          lib,
          config,
          ...
        }:
        let
          cfg = config.hardware.kinect-audio;
          downmixCfg = cfg.downmix;
          pipewireTargetObject =
            if downmixCfg.sourceNodeName != null then downmixCfg.sourceNodeName else downmixCfg.rawSourceName;
        in
        {
          options.hardware.kinect-audio = {
            enable = lib.mkEnableOption "kinect audio support";

            downmix = {
              enable = lib.mkEnableOption "a PipeWire mono source for the Kinect microphone array";

              gain = lib.mkOption {
                type = lib.types.addCheck lib.types.number (gain: gain >= 0);
                default = 0.25;
                example = 0.5;
                description = "Linear gain applied to each of the four microphone channels.";
              };

              sourceNodeName = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                example = "alsa_input.usb-Microsoft_Kinect_for_Windows_USB_Audio_<serial>-02.analog-surround-40";
                description = ''
                  Exact PipeWire node name for the Kinect 4-channel capture source.
                  When unset, WirePlumber renames the detected Kinect source
                  to `rawSourceName` and the loopback uses that stable name.
                '';
              };

              rawSourceName = lib.mkOption {
                type = lib.types.str;
                default = "kinect_array_raw";
                description = ''
                  Stable PipeWire node name assigned to the raw 4-channel
                  Kinect capture source.
                '';
              };

              virtualSourceName = lib.mkOption {
                type = lib.types.str;
                default = "kinect_array_mono";
                description = "PipeWire node name for the mono Kinect microphone source.";
              };

              virtualSourceDescription = lib.mkOption {
                type = lib.types.str;
                default = "Kinect Microphone Array Mono";
                description = "Human-readable description for the mono Kinect microphone source.";
              };
            };
          };

          config = lib.mkIf cfg.enable {
            services.udev.packages = [
              self.packages."${pkgs.system}".kinect-firmware
            ];

            services.pipewire.wireplumber.extraConfig =
              lib.mkIf (downmixCfg.enable && downmixCfg.sourceNodeName == null)
                {
                  "90-kinect-audio-source-name" = {
                    "monitor.alsa.rules" = [
                      {
                        matches = [
                          {
                            "node.name" = "~alsa_input.usb-Microsoft_Kinect_for_Windows_USB_Audio_.*-02.analog-surround-40";
                            "device.vendor.id" = "0x045e";
                            "device.product.id" = "0x02c3";
                          }
                        ];
                        actions.update-props = {
                          "node.name" = downmixCfg.rawSourceName;
                          "node.description" = "Kinect Microphone Array Raw";
                        };
                      }
                    ];
                  };
                };

            services.pipewire.extraConfig.pipewire = lib.mkIf downmixCfg.enable {
              "91-kinect-audio-mono" = {
                "context.modules" = [
                  {
                    name = "libpipewire-module-filter-chain";
                    args = {
                      "node.description" = downmixCfg.virtualSourceDescription;
                      "media.name" = downmixCfg.virtualSourceDescription;

                      "filter.graph" = {
                        nodes = [
                          {
                            type = "builtin";
                            name = "mixer";
                            label = "mixer"; # for builtin this is the filter to use!
                            control = {
                              "Gain 1" = downmixCfg.gain; # FL
                              "Gain 2" = downmixCfg.gain; # FR
                              "Gain 3" = downmixCfg.gain; # RL
                              "Gain 4" = downmixCfg.gain; # RR
                            };
                          }
                        ];
                        inputs = [
                          "mixer:In 1"
                          "mixer:In 2"
                          "mixer:In 3"
                          "mixer:In 4"
                        ];
                        outputs = [ "mixer:Out" ];
                      };

                      "capture.props" = {
                        "target.object" = pipewireTargetObject;
                        "audio.channels" = 4;
                        "audio.position" = [
                          "FL"
                          "FR"
                          "RL"
                          "RR"
                        ];
                        "node.passive" = true;
                        "stream.dont-remix" = true;
                      };

                      "playback.props" = {
                        "node.name" = downmixCfg.virtualSourceName;
                        "node.description" = downmixCfg.virtualSourceDescription;
                        "media.class" = "Audio/Source";
                        "audio.channels" = 1;
                        "audio.position" = [ "MONO" ];
                      };
                    };
                  }
                ];
              };
            };
          };
        };
    };
}
