use log::*;

mod internal;

pub const VENDOR_MICROSOFT: u16 = 0x045e;
pub const PRODUCT_K4W_AUDIO_ORIGINAL: u16 = 0x02be;
pub const KINECT_AUDIO_CONFIGURATION: u8 = 1;
pub const KINECT_AUDIO_INTERFACE: u8 = 0;
pub const KINECT_AUDIO_ENDPOINT_IN: u8 = 0x81;
pub const KINECT_AUDIO_ENDPOINT_OUT: u8 = 0x01;
/// Timeout for each USB transfer, including the device's command acknowledgement.
/// A zero duration would allow a stalled device to block indefinitely.
pub const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Data packet for receiving data
pub struct Response {
    data: [u8; 512],
    len: usize,
}

impl Response {
    fn empty() -> Self {
        Self {
            data: [0u8; 512],
            len: 0,
        }
    }

    pub fn get(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
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

#[repr(u32)]
pub enum Command {
    Status = 0,
    Page = 3,
    Execute = 4,
}

pub fn send(
    device: &rusb::DeviceHandle<rusb::GlobalContext>,
    command: Command,
    tag: u32,
    address: u32,
    data: &[u8],
) -> Result<(), Error> {
    let cmd = internal::Command {
        command: command as u32,
        tag,
        address,
        size: data.len() as u32,
        unk: 0,
    };

    internal::send_command(device, &cmd)?;
    for packet in internal::packets(data) {
        info!(
            "SENDING PACKET - TAG {} - ADDRESS {:x} - PACKET {}",
            tag,
            address,
            packet.len()
        );
        internal::send(device, packet)?;
    }
    let result = internal::receive_status(device)?;

    if result.tag != tag {
        error!("TAG MISMATCH EXPECTED {} -  ACTUAL {}", tag, result.tag);
        return Err(Error::Tag);
    }

    if !result.success {
        error!("STATUSCODE NONZERO");
        return Err(Error::Result);
    }

    Ok(())
}

pub fn receive(
    device: &rusb::DeviceHandle<rusb::GlobalContext>,
    command: Command,
    tag: u32,
    address: u32,
    size: u32,
) -> Result<Response, Error> {
    let cmd = internal::Command {
        command: command as u32,
        tag,
        address,
        size,
        unk: 0,
    };

    internal::send_command(device, &cmd)?;

    info!("RECEIVING RESPONSE");
    let response = internal::receive(device)?;

    let result = internal::receive_status(device)?;

    if response.get().len() != size as usize {
        error!(
            "PAYLOAD SIZE MISMATCH EXPECTED {} - ACTUAL {}",
            size,
            response.get().len()
        );
        return Err(Error::Payload);
    }

    if result.tag != tag {
        error!("TAG MISMATCH EXPECTED {} -  ACTUAL {}", tag, result.tag);
        return Err(Error::Tag);
    }

    if !result.success {
        error!("STATUSCODE NONZERO");
        return Err(Error::Result);
    }

    Ok(response)
}
