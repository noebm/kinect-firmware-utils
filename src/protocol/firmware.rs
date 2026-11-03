use binrw::BinRead;

use super::Error;

/// Firmware version record. Its interpretation follows the experimental status layout.
#[derive(Debug, PartialEq, Eq, BinRead)]
#[br(little)]
pub struct Version {
    pub minor: u16,
    pub major: u16,
    pub release: u32,
    pub patch: u32,
}

impl std::fmt::Display for Version {
    fn fmt(&self, writer: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            writer,
            "{:02}.{:02}.{:02}.{:02}",
            self.major, self.minor, self.release, self.patch,
        )
    }
}

/// Firmware status with experimental version decoding and the complete raw payload.
#[derive(Debug)]
pub struct FirmwareStatus {
    /// The first three records are assumed to use the firmware version layout.
    pub versions: [Version; 3],
    bytes: [u8; 96],
}

impl FirmwareStatus {
    pub(super) fn parse(bytes: &[u8]) -> Result<Self, Error> {
        let bytes: [u8; 96] = bytes.try_into().map_err(|_| Error::Payload)?;
        let mut cursor = std::io::Cursor::new(&bytes);
        let versions = <[Version; 3]>::read(&mut cursor).map_err(|_| Error::Payload)?;

        Ok(Self { versions, bytes })
    }

    /// Returns the complete status payload, including undecoded fields.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_versions_and_preserves_raw_status() {
        let mut bytes = [0xff; 96];
        for (index, chunk) in bytes[..36].chunks_mut(12).enumerate() {
            chunk[..2].copy_from_slice(&(index as u16 + 1).to_le_bytes());
            chunk[2..4].copy_from_slice(&2u16.to_le_bytes());
            chunk[4..8].copy_from_slice(&300u32.to_le_bytes());
            chunk[8..12].copy_from_slice(&400u32.to_le_bytes());
        }

        let status = FirmwareStatus::parse(&bytes).expect("valid status");

        assert_eq!(status.as_bytes(), bytes);
        assert_eq!(status.versions[0].to_string(), "02.01.300.400");
        assert_eq!(status.versions[2].minor, 3);
    }

    #[test]
    fn rejects_invalid_status_lengths() {
        for len in [0, 35, 36, 95, 97] {
            assert!(matches!(
                FirmwareStatus::parse(&vec![0; len]),
                Err(Error::Payload)
            ));
        }
    }
}
