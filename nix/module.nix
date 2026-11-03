{ self }:
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
}
