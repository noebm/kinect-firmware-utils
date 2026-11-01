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

fn main() -> Result<(), Error> {
    let filename = std::env::args()
        .nth(1)
        .unwrap_or("firmware.bin".to_string());

    let firmware: Vec<u8> = std::fs::read(filename).expect("Failed to open file");
    let firmware_header = Header::from_slice(&firmware).expect("Could not parse firmware header");

    println!("FIRMWARE HEADER {}", firmware_header);
    assert_eq!(firmware_header.size, firmware.len() as u32);

    let device = open_audio_device(PRODUCT_K4W_AUDIO_ORIGINAL)?;
    let device = setup_device(device)?;

    let mut seq = 1u32;

    let _firmware_status = receive(&device, Command::Status, seq, 0x15, 0x60)?;

    const PAGESIZE: usize = 0x4000;
    let pages = firmware.chunks(PAGESIZE);
    let addresses = (firmware_header.base_address..).step_by(PAGESIZE);

    for (address, page) in addresses.zip(pages) {
        seq += 1;
        send(&device, Command::Page, seq, address, page)?;
    }

    seq += 1;

    send(
        &device,
        Command::Execute,
        seq,
        firmware_header.entry_point,
        &[],
    )
}
