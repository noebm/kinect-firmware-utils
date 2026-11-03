use kinect_firmware_utils::*;

fn setup_device(
    device: rusb::DeviceHandle<rusb::GlobalContext>,
) -> Result<rusb::DeviceHandle<rusb::GlobalContext>, Error> {
    if device.active_configuration()? != KINECT_AUDIO_CONFIGURATION {
        device.set_active_configuration(KINECT_AUDIO_CONFIGURATION)?;
    }

    device.set_auto_detach_kernel_driver(true)?;
    device.claim_interface(KINECT_AUDIO_INTERFACE)?;

    let actual = device.active_configuration()?;
    if actual != KINECT_AUDIO_CONFIGURATION {
        return Err(Error::Configuration { actual });
    }

    Ok(device)
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Error> {
    let filename = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("firmware.bin"));

    let firmware = std::fs::read(&filename).map_err(|source| Error::FirmwareRead {
        path: filename,
        source,
    })?;
    let firmware_header = Header::from_slice(&firmware)?;

    if firmware.len() != firmware_header.size as usize {
        return Err(Error::FirmwareSize {
            expected: firmware_header.size,
            actual: firmware.len(),
        });
    }

    println!("FIRMWARE HEADER {}", firmware_header);

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

    execute(&device, seq, firmware_header.entry_point)
}
