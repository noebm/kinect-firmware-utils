# kinect-firmware-utils

Implementation based on [kinect-audio-setup](https://git.ao2.it/kinect-audio-setup.git/) and [libfreenect](https://github.com/OpenKinect/libfreenect).

# Notes

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
