use log::*;

mod command;
mod firmware;
mod internal;

pub use firmware::{FirmwareStatus, Version};

const PACKET_SIZE: usize = 512;

pub const VENDOR_MICROSOFT: u16 = 0x045e;
pub const PRODUCT_K4W_AUDIO_ORIGINAL: u16 = 0x02be;
pub const KINECT_AUDIO_CONFIGURATION: u8 = 1;
pub const KINECT_AUDIO_INTERFACE: u8 = 0;
pub const KINECT_AUDIO_ENDPOINT_IN: u8 = 0x81;
pub const KINECT_AUDIO_ENDPOINT_OUT: u8 = 0x01;
/// Timeout for each USB transfer, including the device's command acknowledgement.
/// A zero duration would allow a stalled device to block indefinitely.
pub const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not read firmware '{}': {source}", path.display())]
    FirmwareRead {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse firmware header: {0}")]
    FirmwareHeader(#[from] binrw::Error),
    #[error("firmware size mismatch: header declares {expected} bytes, file contains {actual}")]
    FirmwareSize { expected: u32, actual: usize },
    #[allow(clippy::upper_case_acronyms)]
    #[error("USB operation failed: {0}")]
    USB(#[from] rusb::Error),
    #[error("Kinect audio device {VENDOR_MICROSOFT:04x}:{product:04x} not found")]
    DeviceNotFound { product: u16 },
    #[error("device configuration changed: expected {KINECT_AUDIO_CONFIGURATION}, got {actual}")]
    Configuration { actual: u8 },
    #[error("incomplete USB write: expected {expected} bytes, wrote {actual}")]
    ShortWrite { expected: usize, actual: usize },
    #[error("invalid USB response payload size")]
    Payload,
    #[error("USB response tag does not match the command")]
    Tag,
    #[error("invalid command status response or command failed")]
    Result,
}

/// Opens the first Kinect audio device matching the product ID.
/// The caller is responsible for configuration and interface setup.
pub fn open_audio_device(product: u16) -> Result<rusb::DeviceHandle<rusb::GlobalContext>, Error> {
    let devices = rusb::devices()?;

    for device in devices.iter() {
        let descriptor = device.device_descriptor()?;
        if descriptor.vendor_id() != VENDOR_MICROSOFT || descriptor.product_id() != product {
            continue;
        }

        return Ok(device.open()?);
    }

    Err(Error::DeviceNotFound { product })
}

fn send(
    device: &rusb::DeviceHandle<rusb::GlobalContext>,
    cmd: internal::Packet,
    tag: u32,
    data: &[u8],
) -> Result<(), Error> {
    info!("SENDING {:08x?}", cmd);
    internal::send(device, cmd.as_slice())?;

    for packet in internal::packets(data) {
        info!("SENDING PACKET - TAG {} - PACKET {}", tag, packet.len());
        internal::send(device, packet)?;
    }
    let result = internal::receive_status(device)?;

    result.check(tag)?;

    Ok(())
}

fn receive(
    device: &rusb::DeviceHandle<rusb::GlobalContext>,
    cmd: internal::Packet,
    tag: u32,
    size: usize,
) -> Result<internal::Packet, Error> {
    info!("SENDING {:08x?}", cmd);
    internal::send(device, cmd.as_slice())?;

    info!("RECEIVING RESPONSE");
    let packet = internal::receive(device)?;

    let result = internal::receive_status(device)?;

    if packet.as_slice().len() != size {
        error!(
            "PAYLOAD SIZE MISMATCH EXPECTED {} - ACTUAL {}",
            size,
            packet.as_slice().len()
        );
        return Err(Error::Payload);
    }

    result.check(tag)?;

    Ok(packet)
}

/// Reads firmware status from a claimed audio command interface.
/// The caller supplies a tag and must serialize access with other commands.
pub fn get_firmware_status(
    device: &rusb::DeviceHandle<rusb::GlobalContext>,
    tag: u32,
) -> Result<FirmwareStatus, Error> {
    let packet = command::firmware_status(tag);
    let response = receive(device, packet, tag, command::FIRMWARE_STATUS_SIZE)?;
    FirmwareStatus::parse(response.as_slice())
}

/// Writes a firmware page to an absolute destination address in device memory.
/// Requires a claimed audio command interface in the firmware upload state.
/// The caller supplies a tag and must serialize access with other commands.
pub fn write_page(
    device: &rusb::DeviceHandle<rusb::GlobalContext>,
    tag: u32,
    address: u32,
    data: &[u8],
) -> Result<(), Error> {
    let packet = command::page(tag, address, data.len());
    send(device, packet, tag, data)
}

/// Starts uploaded firmware at its absolute entry point address.
/// Requires a claimed audio command interface and a completed firmware upload.
/// The caller supplies a tag and must serialize access with other commands.
pub fn execute(
    device: &rusb::DeviceHandle<rusb::GlobalContext>,
    tag: u32,
    entry_point: u32,
) -> Result<(), Error> {
    let packet = command::execute(tag, entry_point);
    send(device, packet, tag, &[])
}

