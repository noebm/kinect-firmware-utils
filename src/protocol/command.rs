use super::internal::Packet;

const COMMAND_MAGIC: u32 = 0x0602_2009;

macro_rules! command {
    ($name:ident($($arg:ident: $ty:ty),* $(,)?) => [$($word:expr),* $(,)?]) => {
        pub(super) fn $name($($arg: $ty),*) -> Packet {
            Packet::from_command(&[$($word),*])
        }
    };
}

pub(super) const FIRMWARE_STATUS_SIZE: usize = 0x60;
pub(super) const MOTION_STATE_SIZE: usize = 0x68;

// Status ignores the address; retain the reference query value.
command!(firmware_status(tag: u32) => [
    COMMAND_MAGIC, tag, FIRMWARE_STATUS_SIZE as u32, 0x0000, 0x15, 0,
]);

command!(page(tag: u32, address: u32, data_len: usize) => [
    COMMAND_MAGIC, tag, data_len as u32, 0x0003, address, 0,
]);

command!(execute(tag: u32, entry_point: u32) => [
    COMMAND_MAGIC, tag, 0, 0x0004, entry_point, 0,
]);

// Audio motor commands omit the trailing firmware word.
command!(tilt(tag: u32, tilt: super::Tilt) => [
    COMMAND_MAGIC, tag, 0, 0x803b, tilt.degrees() as u32,
]);

command!(motion_state(tag: u32) => [
    COMMAND_MAGIC, tag, MOTION_STATE_SIZE as u32, 0x8032,
]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tilt;

    fn le_bytes(cmd: &[u32; 6]) -> Vec<u8> {
        let cmd_buffer = cmd
            .iter()
            .map(|arg| arg.to_le_bytes())
            .collect::<Vec<[u8; 4]>>()
            .concat();
        cmd_buffer
    }

    #[test]
    fn generate_motion_state_command() {
        assert_eq!(
            motion_state(0x12345678).as_slice(),
            [0x09, 0x20, 0x02, 0x06, 0x78, 0x56, 0x34, 0x12, 0x68, 0, 0, 0, 0x32, 0x80, 0, 0,]
        );
    }

    #[test]
    fn generate_motor_command() {
        for degrees in [-31i32, 0, 31] {
            let command = tilt(0x12345678, Tilt::new(degrees).expect("valid angle"));
            let mut expected = vec![
                0x09, 0x20, 0x02, 0x06, 0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0, 0x3b, 0x80, 0, 0,
            ];
            expected.extend_from_slice(&degrees.to_le_bytes());

            assert_eq!(command.as_slice(), expected);
        }
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
        let status = firmware_status(seq);

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
        let page = page(seq, address, page_len);

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
        let finished = execute(seq, entry_point);

        assert_eq!(le_bytes(&finished_cmd), finished.as_slice());
    }
}
