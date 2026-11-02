use super::internal::Packet;

pub(super) fn firmware(command: u32, tag: u32, address: u32, size: u32) -> Packet {
    Packet::from_command(&[0x06022009, tag, size, command, address, 0])
}

#[cfg(test)]
mod tests {
    use super::firmware;

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
        let status = firmware(0, seq, 0x15, 0x60);

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
        let page = firmware(3, seq, address, page_len as u32);

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
        let finished = firmware(4, seq, entry_point, 0);

        assert_eq!(le_bytes(&finished_cmd), finished.as_slice());
    }
}
