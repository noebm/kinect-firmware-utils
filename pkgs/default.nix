{
  pkgs,
  wix-extract,
  crane-lib,
}:
rec {
  kinect-firmware-blob = pkgs.callPackage ./kinect-firmware-blob.nix { inherit wix-extract; };
  kinect-firmware-utils = pkgs.callPackage ./kinect-firmware-utils.nix { inherit crane-lib; };
  kinect-udev-rules = pkgs.callPackage ./kinect-udev-rules.nix {
    inherit kinect-firmware-blob kinect-firmware-utils;
  };
  kinect-firmware = pkgs.symlinkJoin {
    name = "kinect-firmware";
    paths = [
      kinect-firmware-blob
      kinect-firmware-utils
      kinect-udev-rules
    ];
  };
}
