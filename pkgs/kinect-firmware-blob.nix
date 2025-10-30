{
  pkgs,
  wix-extract,
}:

with pkgs;
stdenv.mkDerivation {
  pname = "kinect-firmware-blob";
  version = "1.8";

  src = fetchurl {
    url = "https://download.microsoft.com/download/E/C/5/EC50686B-82F4-4DBF-A922-980183B214E6/KinectRuntime-v1.8-Setup.exe";
    hash = "sha256-9NQUP7DwqNJ2iJwHe/yK9Cv+mcEoytq14xa/AVqYWOk=";
  };
  buildInputs = [ p7zip ];

  unpackPhase = ''
    ${wix-extract} $src -d $TMP
    7z e -y -r $TMP/KinectDrivers-v1.8-x86.WHQL.msi "UACFirmware" > /dev/null
  '';

  installPhase = ''
    install -Dm644 UACFirmware "$out/share/kinect-audio.fw"
  '';
}
