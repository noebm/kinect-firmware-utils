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

    pub(super) fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }
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
}
