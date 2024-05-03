mod header;
mod protocol;
use header::Header;
use protocol::*;

fn setup_device() -> Option<rusb::DeviceHandle<rusb::GlobalContext>> {
    let mut device =
        rusb::open_device_with_vid_pid(VENDOR_MICROSOFT, PRODUCT_K4W_AUDIO_ORIGINAL).unwrap();

    if device.active_configuration().unwrap() != KINECT_AUDIO_CONFIGURATION {
        device
            .set_active_configuration(KINECT_AUDIO_CONFIGURATION)
            .unwrap();
    }

    device.set_auto_detach_kernel_driver(true).unwrap();

    device.claim_interface(KINECT_AUDIO_INTERFACE).unwrap();

    if device.active_configuration().unwrap() != KINECT_AUDIO_CONFIGURATION {
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

    let device = setup_device().unwrap();

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

    send(&device, Command::Execute, seq, firmware_header.entry_point, &[])
}
