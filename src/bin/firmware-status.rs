use kinect_firmware_utils::*;

use binrw::BinRead;
use std::fmt;

// Assumes Version information is structured the same way as in Header.
// With the difference that the release and patch fields are 32 bit.
#[derive(Debug, PartialEq, Eq, BinRead)]
#[br(little)]
pub struct Version {
    minor: u16,
    major: u16,
    release: u32,
    patch: u32,
}

impl fmt::Display for Version {
    fn fmt(&self, writer: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            writer,
            "{:02}.{:02}.{:02}.{:02}",
            self.major, self.minor, self.release, self.patch,
        )
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let device = rusb::open_device_with_vid_pid(VENDOR_MICROSOFT, 0x02c3).unwrap();

    let tag = 0x1337;

    // address is irrelevant
    // response length is always 0x60 irrespective of passed size
    let firmware_status = receive(&device, Command::Status, tag, 0x00, 0x60)?;
    let bytes = firmware_status.get();
    println!("FIRMWARE STATUS:");
    println!("LENGTH: {} / 0x{:x}", bytes.len(), bytes.len());
    for chunk in bytes.chunks(8) {
        for byte in chunk {
            print!("{byte:02x} ");
        }
        println!();
    }

    // Experimental version parsing
    {
        let mut cursor = std::io::Cursor::new(bytes);

        let version1 = Version::read(&mut cursor).unwrap();
        let version2 = Version::read(&mut cursor).unwrap();
        let version3 = Version::read(&mut cursor).unwrap();
        println!("version1: {version1}");
        println!("version2: {version2}");
        println!("version3: {version3}");
    }

    Ok(())
}
