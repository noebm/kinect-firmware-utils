# Notes

## Firmware versions

- 01.02.390.00:
        Source: [KinectSDK-v1.0-beta2-x86.msi](http://download.microsoft.com/download/F/9/9/F99791F2-D5BE-478A-B77A-830AD14950C3/KinectSDK-v1.0-beta2-x86.msi)
        Directly visible as `UACFirmware.C9C6E852_35A3_41DC_A57D_BDDEB43DFD04` after extraction via 7zip.

- 01.02.709.00:
        Source: [KinectSDK-v1.8-Setup.exe](https://www.microsoft.com/en-us/download/details.aspx?id=40278)
        Requires `cabextract` of the exe and afterwards the file `a0` (or `KinectDrivers-v1.8-x64.WHQL.msi` by the xml named `0`), which extracts `UACFirmware`.

- 01.02.810.00:
        Source: [SystemUpdate_17599_USB.zip](https://web.archive.org/web/20220113165637/https://download.microsoft.com/download/b/5/b/b5b2e1bc-a5c7-4e78-9518-e1c59ff738d0/SystemUpdate_17559_USB.zip)
        Requires extraction via fwfetcher.py in libfreenect (PIR archive).

## Usb protocol

Apparently the [usb protocol](https://github.com/microsoft/Azure-Kinect-Sensor-SDK/blob/develop/src/usbcommand/usbcommand.c#L598) for Azure Kinect Sensor is mostly the same.

