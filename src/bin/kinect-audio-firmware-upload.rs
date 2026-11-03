use anyhow::{bail, Context};
use kinect_firmware_utils::Error as ProtocolError;
use kinect_firmware_utils::*;

mod logging;

fn setup_device(
    device: rusb::DeviceHandle<rusb::GlobalContext>,
) -> anyhow::Result<rusb::DeviceHandle<rusb::GlobalContext>> {
    if device.active_configuration()? != KINECT_AUDIO_CONFIGURATION {
        device.set_active_configuration(KINECT_AUDIO_CONFIGURATION)?;
    }

    device.set_auto_detach_kernel_driver(true)?;
    device.claim_interface(KINECT_AUDIO_INTERFACE)?;

    let actual = device.active_configuration()?;
    if actual != KINECT_AUDIO_CONFIGURATION {
        return Err(ProtocolError::Configuration { actual }.into());
    }

    Ok(device)
}

fn main() -> anyhow::Result<()> {
    logging::init();

    let filename = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("firmware.bin"));

    log::info!(
        "Starting Kinect audio firmware upload from '{}'",
        filename.display()
    );

    let firmware = std::fs::read(&filename)
        .with_context(|| format!("could not read firmware '{}'", filename.display()))?;
    let firmware_header =
        Header::from_slice(&firmware).context("could not parse firmware header")?;

    if firmware.len() != firmware_header.size as usize {
        bail!(
            "firmware size mismatch: header declares {} bytes, file contains {}",
            firmware_header.size,
            firmware.len(),
        );
    }

    log::debug!("FIRMWARE HEADER {}", firmware_header);

    let device = open_audio_device(PRODUCT_K4W_AUDIO_ORIGINAL)?;
    let device = setup_device(device)?;

    let mut seq = 1u32;

    let _firmware_status = get_firmware_status(&device, seq)?;

    const PAGESIZE: usize = 0x4000;
    let pages = firmware.chunks(PAGESIZE);
    let addresses = (firmware_header.base_address..).step_by(PAGESIZE);

    for (address, page) in addresses.zip(pages) {
        seq += 1;
        write_page(&device, seq, address, page)?;
    }

    seq += 1;

    execute(&device, seq, firmware_header.entry_point)?;

    log::info!("Kinect audio firmware upload completed successfully");

    Ok(())
}
