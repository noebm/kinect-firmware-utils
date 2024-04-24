use binrw::BinRead;

pub const VENDOR_MICROSOFT: u16 = 0x045e;
pub const PRODUCT_K4W_AUDIO_ORIGINAL: u16 = 0x02be;
pub const KINECT_AUDIO_CONFIGURATION: u8 = 1;
pub const KINECT_AUDIO_INTERFACE: u8 = 0;
pub const KINECT_AUDIO_ENDPOINT_IN: u8 = 0x81;
pub const KINECT_AUDIO_ENDPOINT_OUT: u8 = 0x01;
pub const TIMEOUT: std::time::Duration = std::time::Duration::ZERO;

pub fn command(device: &rusb::DeviceHandle<rusb::GlobalContext>, cmd: &command::Command) {
    println!("COMMAND STATUS {:08x?}", cmd);
    let cmd_buffer = command::serialize(cmd);

    device
        .write_bulk(KINECT_AUDIO_ENDPOINT_OUT, &cmd_buffer, TIMEOUT)
        .unwrap();
}

pub mod command {
    use binrw::BinWrite;

    #[derive(Debug, BinWrite)]
    #[bw(little, magic = b"\x09\x20\x02\x06")]
    pub struct Command {
        tag: u32,
        size: u32,
        command: u32,
        address: u32,
        unk: u32,
    }

    pub fn serialize(cmd: &Command) -> Vec<u8> {
        let mut writer = std::io::Cursor::new(Vec::with_capacity(6 * 4));
        cmd.write(&mut writer).unwrap();
        writer.into_inner()
    }

    #[cfg(test)]
    fn le_bytes(cmd: &[u32; 6]) -> Vec<u8> {
        let cmd_buffer = cmd
            .iter()
            .map(|arg| arg.to_le_bytes())
            .collect::<Vec<[u8; 4]>>()
            .concat();
        cmd_buffer
    }

    pub fn status(tag: u32) -> Command {
        Command {
            tag,
            command: 0,
            address: 0x15,
            size: 0x60,
            unk: 0,
        }
    }

    pub fn page(tag: u32, address: u32, size: u32) -> Command {
        Command {
            tag,
            command: 3,
            address,
            size,
            unk: 0,
        }
    }
    pub fn finished(tag: u32, entry_point: u32) -> Command {
        Command {
            tag,
            command: 0x04,
            address: entry_point,
            size: 0,
            unk: 0,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

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

            assert_eq!(le_bytes(&status_cmd), serialize(&status(seq)));
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

            assert_eq!(
                le_bytes(&page_cmd),
                serialize(&page(seq, address, page_len as u32))
            );
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

            assert_eq!(
                le_bytes(&finished_cmd),
                serialize(&finished(seq, entry_point))
            );
        }
    }
}

pub fn status(device: &rusb::DeviceHandle<rusb::GlobalContext>, seq: u32) -> bool {
    let mut status_buffer = [0u8; 512];

    let status_len = device
        .read_bulk(KINECT_AUDIO_ENDPOINT_IN, &mut status_buffer, TIMEOUT)
        .unwrap();

    get_status(&status_buffer[..status_len], seq)
}

fn get_status(buffer: &[u8], tag: u32) -> bool {
    assert_eq!(buffer.len(), 12);
    let status = Status::read(&mut std::io::Cursor::new(buffer)).unwrap();
    assert_eq!(status.tag, tag);
    status.success
}

#[derive(BinRead)]
#[br(little, magic = b"\x00\xe0\x6f\x0a")]
struct Status {
    tag: u32,

    #[br(map = |x: u32| x == 0)]
    success: bool,
}

#[cfg(test)]
mod tests {
    use super::get_status;

    #[test]
    fn parse_status() {
        let buffer = [0x0a6f_e000, 0x0000_0001, 0x0000_0000u32]
            .iter()
            .map(|arg| arg.to_le_bytes())
            .collect::<Vec<[u8; 4]>>()
            .concat();

        assert!(get_status(&buffer, 1));
    }
}

pub fn response(device: &rusb::DeviceHandle<rusb::GlobalContext>) {
    let mut buffer = [0u8; 512];

    let len = device
        .read_bulk(KINECT_AUDIO_ENDPOINT_IN, &mut buffer, TIMEOUT)
        .unwrap();

    println!("RESPONSE LEN {:#x}", len);

    // for status command
    assert_eq!(len, 0x60);
}
