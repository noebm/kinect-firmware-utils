use kinect_firmware_utils::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let device = open_audio_device(0x02c3)?;

    // Claim the command interface without changing configuration or detaching a driver.
    device.claim_interface(KINECT_AUDIO_INTERFACE)?;

    let tag = 0x1337;

    let firmware_status = get_firmware_status(&device, tag)?;
    let bytes = firmware_status.as_bytes();
    println!("FIRMWARE STATUS:");
    println!("LENGTH: {} / 0x{:x}", bytes.len(), bytes.len());
    for chunk in bytes.chunks(8) {
        for byte in chunk {
            print!("{byte:02x} ");
        }
        println!();
    }

    for (index, version) in firmware_status.versions.iter().enumerate() {
        println!("version{}: {version}", index + 1);
    }

    Ok(())
}
