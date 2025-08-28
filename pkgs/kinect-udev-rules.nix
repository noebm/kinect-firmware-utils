{
  pkgs,
  kinect-firmware-blob,
  kinect-firmware-utils,
}:
with pkgs;
stdenv.mkDerivation rec {
  name = "kinect-udev-rules";

  src = ../udev;
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
}
