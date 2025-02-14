use super::Error;
use super::Response;
use super::{KINECT_AUDIO_ENDPOINT_IN, KINECT_AUDIO_ENDPOINT_OUT, TIMEOUT};
use binrw::{BinRead, BinWrite};
use log::*;

pub fn send_command(
    device: &rusb::DeviceHandle<rusb::GlobalContext>,
    cmd: &Command,
) -> Result<(), Error> {
    info!("SENDING {:08x?}", cmd);
    let cmd_buffer = cmd.bytes();

    device
        .write_bulk(KINECT_AUDIO_ENDPOINT_OUT, &cmd_buffer, TIMEOUT)
        .map_err(Error::USB)?;
    Ok(())
}

#[derive(Debug, BinWrite)]
#[bw(little, magic = b"\x09\x20\x02\x06")]
pub struct Command {
    pub tag: u32,
    pub size: u32,
    pub command: u32,
    pub address: u32,
    pub unk: u32,
}

impl Command {
    fn bytes(&self) -> [u8; 24] {
        let mut writer = std::io::Cursor::new([0u8; 24]);
        self.write(&mut writer).unwrap();
        writer.into_inner()
    }
}

pub fn receive_status(device: &rusb::DeviceHandle<rusb::GlobalContext>) -> Result<Status, Error> {
    info!("RECEIVING STATUS RESULT");
    let response = receive(device)?;
    response.try_into().map_err(|_| Error::Result)
}

impl TryFrom<Response> for Status {
    type Error = ();
    fn try_from(value: Response) -> Result<Self, Self::Error> {
        if value.get().len() != 12 {
            return Err(());
        }
        Self::read(&mut std::io::Cursor::new(value.get())).map_err(|_| ())
    }
}

#[derive(BinRead)]
#[br(little, magic = b"\x00\xe0\x6f\x0a")]
pub struct Status {
    pub tag: u32,

    #[br(map = |x: u32| x == 0)]
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::Command;

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
        let status = Command {
            tag: seq,
            command: 0,
            address: 0x15,
            size: 0x60,
            unk: 0,
        };

        assert_eq!(le_bytes(&status_cmd), status.bytes());
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
        let page = Command {
            tag: seq,
            command: 3,
            address,
            size: page_len as u32,
            unk: 0,
        };

        assert_eq!(le_bytes(&page_cmd), page.bytes());
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
        let finished = Command {
            tag: seq,
            command: 0x04,
            address: entry_point,
            size: 0,
            unk: 0,
        };

        assert_eq!(le_bytes(&finished_cmd), finished.bytes());
    }

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

/// Data packet for sending data
pub struct Packet<'a>(&'a [u8]);

impl<'a> Packet<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub fn send(device: &rusb::DeviceHandle<rusb::GlobalContext>, packet: Packet) {
    device
        .write_bulk(KINECT_AUDIO_ENDPOINT_OUT, packet.0, TIMEOUT)
        .unwrap();
}

pub struct Packets<'a>(core::slice::Chunks<'a, u8>);

const PACKET_SIZE: usize = 512;

impl<'a> From<&'a [u8]> for Packets<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self(value.chunks(PACKET_SIZE))
    }
}

pub fn packets(data: &[u8]) -> Packets {
    data.into()
}

impl<'a> std::iter::Iterator for Packets<'a> {
    type Item = Packet<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(Packet(self.0.next()?))
    }
}

pub fn receive(device: &rusb::DeviceHandle<rusb::GlobalContext>) -> Result<Response, Error> {
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
