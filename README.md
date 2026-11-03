# kinect-firmware-utils

Utilities for interacting with the Kinect audio interface, including uploading
the firmware required to use the Kinect as a microphone.

Implementation based on [kinect-audio-setup](https://git.ao2.it/kinect-audio-setup.git/) and [libfreenect](https://github.com/OpenKinect/libfreenect).

The package provides three commands:

- `kinect-audio-firmware-upload [<firmware.bin>]`: upload Kinect audio firmware.
- `kinect-audio-firmware-status`: query audio firmware status.
- `kinect-tilt [<degrees>]`: read or set the tilt angle.

## Device support

The utilities target Kinect for Windows / model 1473 audio devices with
Microsoft USB vendor ID `045e`:

- `045e:02be`: audio device before firmware upload.
- `045e:02c3`: running audio device, used for firmware status and tilt control.

The commands require USB access to the corresponding device.

# Notes

## Firmware upload logs

The udev rule sends firmware uploader output to the system journal, including
start and successful completion messages at info priority and errors at error
priority. Set `RUST_LOG=debug` when running the uploader manually to include
firmware header and USB protocol details.
Completion means the device acknowledged the firmware execution command.
View uploads and errors from the current boot:

```sh
journalctl -b -t kinect-audio-firmware-upload
```

## Firmware versions

- 01.02.390.00:
        Source: [KinectSDK-v1.0-beta2-x86.msi](http://download.microsoft.com/download/F/9/9/F99791F2-D5BE-478A-B77A-830AD14950C3/KinectSDK-v1.0-beta2-x86.msi)
        Directly visible as `UACFirmware.C9C6E852_35A3_41DC_A57D_BDDEB43DFD04` after extraction via 7zip.

- 01.02.709.00:
        Source: [KinectSDK-v1.8-Setup.exe](https://download.microsoft.com/download/E/1/D/E1DEC243-0389-4A23-87BF-F47DE869FC1A/KinectSDK-v1.8-Setup.exe)
        The file is a wix executable which can be extracted.
        The firmware is contained in the file named `UACFirmware` inside `KinectDrivers-v1.8-{x64,x86}.WHQL.msi`.

- 01.02.810.00:
        Source: [SystemUpdate_17599_USB.zip](https://web.archive.org/web/20220113165637/https://download.microsoft.com/download/b/5/b/b5b2e1bc-a5c7-4e78-9518-e1c59ff738d0/SystemUpdate_17559_USB.zip)
        Requires extraction via fwfetcher.py in libfreenect (PIR archive).

## Usb protocol

Apparently the [usb protocol](https://github.com/microsoft/Azure-Kinect-Sensor-SDK/blob/develop/src/usbcommand/usbcommand.c#L598) for Azure Kinect Sensor is mostly the same.

## PipeWire microphone array downmix

The Kinect for Windows audio firmware exposes the microphone array as a
4-channel, 16 kHz, 32-bit capture device. Some desktop applications handle that
as a surround input instead of a microphone array, which can make the usable
voice level too low.

The NixOS module can create a virtual mono PipeWire source by downmixing the
four Kinect channels:

```nix
{
  imports = [ inputs.kinect-firmware-utils.nixosModules.default ];

  hardware.kinect-audio = {
    enable = true;

    downmix = {
      enable = true;
      gain = 0.25;

      # Optional fallback if WirePlumber does not detect the Kinect source:
      # sourceNodeName = "alsa_input.usb-Microsoft_Kinect_for_Windows_USB_Audio_<serial>-02.analog-surround-40";
    };
  };
}
```
