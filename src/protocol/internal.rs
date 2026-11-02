use super::{
    Error, Response, KINECT_AUDIO_ENDPOINT_IN, KINECT_AUDIO_ENDPOINT_OUT, PACKET_SIZE, TIMEOUT,
};
use binrw::BinRead;
use log::*;

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

impl TryFrom<Response> for Status {
    type Error = Error;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        if value.get().len() != 12 {
            return Err(Error::Result);
        }

        Self::read(&mut std::io::Cursor::new(value.get())).map_err(|_| Error::Result)
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

pub(super) fn receive(device: &rusb::DeviceHandle<rusb::GlobalContext>) -> Result<Response, Error> {
    let mut packet = Response::empty();

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
    use super::{Response, Status};

    #[test]
    fn parse_status() {
        let buffer = [0x0a6f_e000, 0x0000_0001, 0x0000_0000u32]
            .iter()
            .map(|arg| arg.to_le_bytes())
            .collect::<Vec<[u8; 4]>>()
            .concat();

        let mut response = Response::empty();
        response.data[..buffer.len()].copy_from_slice(&buffer);
        response.len = buffer.len();

        let status: Status = response.try_into().unwrap();
        assert_eq!(status.tag, 1);
        assert!(status.success);
    }
}
