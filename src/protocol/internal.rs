use super::{Error, KINECT_AUDIO_ENDPOINT_IN, KINECT_AUDIO_ENDPOINT_OUT, PACKET_SIZE, TIMEOUT};
use binrw::BinRead;
use log::*;

pub(super) struct Packet {
    data: [u8; PACKET_SIZE],
    len: usize,
}

impl Packet {
    pub(super) fn from_command<const N: usize>(words: &[u32; N]) -> Self {
        const { assert!(N <= PACKET_SIZE / size_of::<u32>()) };

        let mut packet = Self::empty();
        packet.len = N * size_of::<u32>();

        for (chunk, word) in packet.data[..packet.len].chunks_mut(4).zip(words) {
            chunk.copy_from_slice(&word.to_le_bytes());
        }

        packet
    }

    fn empty() -> Self {
        Self {
            data: [0; PACKET_SIZE],
            len: 0,
        }
    }

    pub(super) fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

impl std::fmt::Debug for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Packet")
            .field("data", &self.as_slice())
            .finish()
    }
}

#[derive(BinRead)]
#[br(little, magic = b"\x00\xe0\x6f\x0a")]
pub(super) struct Status {
    tag: u32,

    #[br(map = |x: u32| x == 0)]
    success: bool,
}

impl Status {
    pub(super) fn check(self, tag: u32) -> Result<(), Error> {
        if self.tag != tag {
            error!("TAG MISMATCH EXPECTED {} - ACTUAL {}", tag, self.tag);
            return Err(Error::Tag);
        }

        if !self.success {
            error!("STATUSCODE NONZERO");
            return Err(Error::Result);
        }

        Ok(())
    }
}

impl TryFrom<Packet> for Status {
    type Error = Error;

    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        if value.as_slice().len() != 12 {
            return Err(Error::Result);
        }

        Self::read(&mut std::io::Cursor::new(value.as_slice())).map_err(|_| Error::Result)
    }
}

pub(super) fn receive_status(
    device: &rusb::DeviceHandle<rusb::GlobalContext>,
) -> Result<Status, Error> {
    info!("RECEIVING STATUS RESULT");
    let response = receive(device)?;
    response.try_into()
}

pub(super) fn send(
    device: &rusb::DeviceHandle<rusb::GlobalContext>,
    packet: &[u8],
) -> Result<(), Error> {
    let actual = device
        .write_bulk(KINECT_AUDIO_ENDPOINT_OUT, packet, TIMEOUT)
        .map_err(Error::USB)?;

    if actual != packet.len() {
        return Err(Error::ShortWrite {
            expected: packet.len(),
            actual,
        });
    }

    Ok(())
}

pub(super) fn packets(data: &[u8]) -> core::slice::Chunks<'_, u8> {
    data.chunks(PACKET_SIZE)
}

pub(super) fn receive(device: &rusb::DeviceHandle<rusb::GlobalContext>) -> Result<Packet, Error> {
    let mut packet = Packet::empty();

    let len = device
        .read_bulk(KINECT_AUDIO_ENDPOINT_IN, &mut packet.data, TIMEOUT)
        .map_err(Error::USB)?;

    if len > packet.data.len() {
        return Err(Error::Payload);
    }

    packet.len = len;
    Ok(packet)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test fixtures must parse successfully")]
mod tests {
    use super::{Packet, Status};

    #[test]
    fn command_words_are_little_endian() {
        let packet = Packet::from_command(&[0x1234_5678, 0xaabb_ccdd]);

        assert_eq!(
            packet.as_slice(),
            [0x78, 0x56, 0x34, 0x12, 0xdd, 0xcc, 0xbb, 0xaa]
        );
    }

    #[test]
    fn command_can_fill_packet() {
        let packet = Packet::from_command(&[0x1234_5678; super::PACKET_SIZE / 4]);

        assert_eq!(packet.as_slice().len(), super::PACKET_SIZE);
        assert!(packet
            .as_slice()
            .chunks_exact(4)
            .all(|chunk| chunk == [0x78, 0x56, 0x34, 0x12]));
    }

    #[test]
    fn parse_status() {
        let response = Packet::from_command(&[0x0a6f_e000, 1, 0]);

        let status: Status = response.try_into().unwrap();
        assert_eq!(status.tag, 1);
        assert!(status.success);
    }
}
