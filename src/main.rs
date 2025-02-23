use kinect_firmware_utils::*;

fn setup_device(
    mut device: rusb::DeviceHandle<rusb::GlobalContext>,
) -> Option<rusb::DeviceHandle<rusb::GlobalContext>> {
    if device.active_configuration().ok()? != KINECT_AUDIO_CONFIGURATION {
        device
            .set_active_configuration(KINECT_AUDIO_CONFIGURATION)
            .ok()?;
    }

    device.set_auto_detach_kernel_driver(true).ok()?;

    device.claim_interface(KINECT_AUDIO_INTERFACE).ok()?;

    if device.active_configuration().ok()? != KINECT_AUDIO_CONFIGURATION {
        println!("device configuration changed!");
        return None;
    }
    Some(device)
}

fn main() -> Result<(), Error> {
    let filename = std::env::args()
        .nth(1)
        .unwrap_or("firmware.bin".to_string());

    let firmware: Vec<u8> = std::fs::read(filename).expect("Failed to open file");
    let firmware_header = Header::from_slice(&firmware).expect("Could not parse firmware header");

    println!("FIRMWARE HEADER {}", firmware_header);
    assert_eq!(firmware_header.size, firmware.len() as u32);

    let device = {
        let Some(device) =
            rusb::open_device_with_vid_pid(VENDOR_MICROSOFT, PRODUCT_K4W_AUDIO_ORIGINAL)
        else {
            println!("Device not found. Exiting..");
            return Ok(());
        };

        setup_device(device).expect("Failed to initialize device.")
    };

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
