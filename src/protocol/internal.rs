use super::{
    Error, Response, KINECT_AUDIO_ENDPOINT_IN, KINECT_AUDIO_ENDPOINT_OUT, PACKET_SIZE, TIMEOUT,
};
use binrw::BinRead;
use log::*;

#[derive(Debug)]
pub(super) struct Command {
    data: [u8; 24],
    len: usize,
}

impl Command {
    pub(super) fn firmware(command: u32, tag: u32, address: u32, size: u32) -> Self {
        let words = [0x06022009, tag, size, command, address, 0];
        let mut data = [0; 24];

        for (chunk, word) in data.chunks_mut(4).zip(words) {
            chunk.copy_from_slice(&word.to_le_bytes());
        }

        Self { data, len: 24 }
    }

    fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
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

impl TryFrom<Response> for Status {
    type Error = Error;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        if value.get().len() != 12 {
            return Err(Error::Result);
        }

        Self::read(&mut std::io::Cursor::new(value.get())).map_err(|_| Error::Result)
    }
}

pub(super) fn send_command(
    device: &rusb::DeviceHandle<rusb::GlobalContext>,
    cmd: &Command,
) -> Result<(), Error> {
    info!("SENDING {:08x?}", cmd);
    send(device, cmd.as_slice())
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
    use super::{Command, Response, Status};

    fn le_bytes(cmd: &[u32; 6]) -> Vec<u8> {
        let cmd_buffer = cmd
            .iter()
            .map(|arg| arg.to_le_bytes())
            .collect::<Vec<[u8; 4]>>()
            .concat();
        cmd_buffer
    }

    #[test]
    fn generate_status_command() {
        let seq = 1;
        let status_cmd = [
            0x06022009u32, // magic
            seq,           // tag
            0x00000060u32, // payload size
            0x00000000u32, // command = version
            0x00000015u32, // address
            0x00000000u32, // unk
        ];
        let status = Command::firmware(0, seq, 0x15, 0x60);

        assert_eq!(le_bytes(&status_cmd), status.as_slice());
    }

    #[test]
    fn generate_page_command() {
        let seq = 7;
        let page_len: usize = 0x10_000;
        let address = 0x80_000;
        let page_cmd = [
            0x06022009u32,     // magic
            seq,               // tag
            (page_len as u32), // payload size
            0x00000003u32,     // command = write page
            address,           // address
            0x00000000u32,     // unk
        ];
        let page = Command::firmware(3, seq, address, page_len as u32);

        assert_eq!(le_bytes(&page_cmd), page.as_slice());
    }

    #[test]
    fn generate_finished_command() {
        let seq = 11;
        let entry_point = 0x80_030;
        let finished_cmd = [
            0x0602_2009u32, // magic
            seq,            // tag
            0x0000_0000u32, // payload size
            0x0000_0004u32, // command = finish upload
            entry_point,    // address
            0x0000_0000u32, // unk
        ];
        let finished = Command::firmware(4, seq, entry_point, 0);

        assert_eq!(le_bytes(&finished_cmd), finished.as_slice());
    }

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
