use std::fmt;

const MAGIC: u32 = 0xca77f00d;

#[repr(C)]
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
        )?;
        Ok(())
    }
}

#[repr(C)]
pub struct Header {
    pub version: Version,
    pub base_address: u32, // Base address of firmware image.  2BL starts at 0x10000, audios starts at 0x80000.
    pub size: u32,         // Size of firmware image, in bytes
    pub entry_point: u32,  // Code entry point (absolute address)
}

impl fmt::Display for Header {
    fn fmt(&self, writer: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(writer, "Found firmware image:\n")?;
        write!(writer, "\tversion      {}\n", self.version)?;
        write!(writer, "\tbase address {:#08x}\n", self.base_address)?;
        write!(writer, "\tsize         {:#08x}\n", self.size)?;
        write!(writer, "\tentry point  {:#08x}\n", self.entry_point)?;
        Ok(())
    }
}

impl Header {
    pub fn from_slice(buffer: &[u8]) -> Option<Self> {
        let magic = u32::from_le_bytes(buffer[0..4].try_into().ok()?);

        if magic != MAGIC {
            return None;
        }

        let version = Version {
            minor: u16::from_le_bytes(buffer[04..06].try_into().ok()?),
            major: u16::from_le_bytes(buffer[06..08].try_into().ok()?),
            release: u16::from_le_bytes(buffer[08..10].try_into().ok()?),
            patch: u16::from_le_bytes(buffer[10..12].try_into().ok()?),
        };

        let base_address = u32::from_le_bytes(buffer[12..16].try_into().ok()?);
        let size = u32::from_le_bytes(buffer[16..20].try_into().ok()?);
        let entry_point = u32::from_le_bytes(buffer[20..24].try_into().ok()?);

        Some(Self {
            version,
            base_address,
            size,
            entry_point,
        })
    }
}
