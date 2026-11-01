use binrw::BinRead;
use std::fmt;

#[derive(Debug, PartialEq, Eq, BinRead)]
#[br(little)]
pub struct Version {
    minor: u16,   // The version string has four parts, each a 16-bit little-endian int.
    major: u16,   // Yes, minor comes before major.
    release: u16, //
    patch: u16,   //
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

#[derive(BinRead)]
#[br(little, magic = b"\x0d\xf0\x77\xca")]
pub struct Header {
    pub version: Version,
    pub base_address: u32, // Base address of firmware image.  2BL starts at 0x10000, audios starts at 0x80000.
    pub size: u32,         // Size of firmware image, in bytes
    pub entry_point: u32,  // Code entry point (absolute address)
}

impl fmt::Display for Header {
    fn fmt(&self, writer: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(
            writer,
            "version      {}\nbase address {:#08x}\nsize         {:#08x}\nentry point  {:#08x}",
            self.version, self.base_address, self.size, self.entry_point
        )
    }
}

impl Header {
    pub fn from_slice(buffer: &[u8]) -> binrw::BinResult<Self> {
        Self::read(&mut std::io::Cursor::new(buffer))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn reject_invalid_magic() {
        assert!(matches!(
            Header::from_slice(&[0; 24]),
            Err(binrw::Error::BadMagic { .. })
        ));
    }

    #[test]
    fn reject_truncated_header() {
        let data = b"\x0d\xf0\x77\xca";
        assert!(matches!(Header::from_slice(data), Err(error) if error.is_eof()));
    }

    #[test]
    fn parse_header() {
        let data = b"\x0d\xf0\x77\xca\x02\x00\x01\x00\x86\x01\x00\x00\x00\x00\x08\x00\x00\x04\x02\x00\x30\x00\x08\x00";
        println!("{:x?}", data);
        let magic = u32::from_le_bytes(data[0..4].try_into().unwrap());
        println!("magic: {magic:x}");

        let header = Header::from_slice(data).unwrap();

        assert_eq!(
            header.version,
            Version {
                minor: 2,
                major: 1,
                release: 390,
                patch: 0
            }
        );
        assert_eq!(header.base_address, 0x80_000);
        assert_eq!(header.size, 0x20_400);
        assert_eq!(header.entry_point, 0x80_030);
    }
}
