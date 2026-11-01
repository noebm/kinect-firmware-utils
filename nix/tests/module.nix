{
  nixpkgs,
  self,
  system,
}:
let
  pkgs = nixpkgs.legacyPackages.${system};
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
  ''
