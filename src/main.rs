const VENDOR_MICROSOFT: u16 = 0x045e;
const PRODUCT_K4W_AUDIO_ORIGINAL: u16 = 0x02be;
const KINECT_AUDIO_CONFIGURATION: u8 = 1;
const KINECT_AUDIO_INTERFACE: u8 = 0;
const KINECT_AUDIO_ENDPOINT_IN: u8 = 0x81;
const KINECT_AUDIO_ENDPOINT_OUT: u8 = 0x01;
const TIMEOUT: std::time::Duration = std::time::Duration::ZERO;

fn status(device: &rusb::DeviceHandle<rusb::GlobalContext>, cmd: &[u32], seq: u32) -> bool {
    let mut status_buffer = [0u8; 512];
    println!("STATUS {:08x?}", cmd);

    let status_len = device
        .read_bulk(KINECT_AUDIO_ENDPOINT_IN, &mut status_buffer, TIMEOUT)
        .unwrap();

    assert_eq!(status_len, 12);
    assert_eq!(
        u32::from_le_bytes(status_buffer[0..04].try_into().unwrap()),
        0x0a6f_e000
    );
    assert_eq!(
        u32::from_le_bytes(status_buffer[4..08].try_into().unwrap()),
        seq
    );
    u32::from_le_bytes(status_buffer[8..12].try_into().unwrap()) == 0
}

fn response(device: &rusb::DeviceHandle<rusb::GlobalContext>) {
    let mut buffer = [0u8; 512];

    let len = device
        .read_bulk(KINECT_AUDIO_ENDPOINT_IN, &mut buffer, TIMEOUT)
        .unwrap();

    println!("RESPONSE LEN {:#x}", len);

    // for status command
    assert_eq!(len, 0x60);
}

fn command(device: &rusb::DeviceHandle<rusb::GlobalContext>, cmd: &[u32]) {
    println!("COMMAND STATUS {:08x?}", cmd);
    let cmd_buffer = cmd
        .iter()
        .map(|arg| arg.to_le_bytes())
        .collect::<Vec<[u8; 4]>>()
        .concat();

    device
        .write_bulk(KINECT_AUDIO_ENDPOINT_OUT, &cmd_buffer, TIMEOUT)
        .unwrap();
}

fn main() {
    let filename = std::env::args()
        .nth(1)
        .unwrap_or("firmware.bin".to_string());

    let device = {
        let mut device =
            rusb::open_device_with_vid_pid(VENDOR_MICROSOFT, PRODUCT_K4W_AUDIO_ORIGINAL).unwrap();

        if device.active_configuration().unwrap() != KINECT_AUDIO_CONFIGURATION {
            device
                .set_active_configuration(KINECT_AUDIO_CONFIGURATION)
                .unwrap();
        }

        device.set_auto_detach_kernel_driver(true).unwrap();

        device.claim_interface(KINECT_AUDIO_INTERFACE).unwrap();

        if device.active_configuration().unwrap() != KINECT_AUDIO_CONFIGURATION {
            println!("device configuration changed!");
            return;
        }
        device
    };

    let mut seq = 1u32;

    // write command
    let status_cmd = &[
        0x06022009u32, // magic
        seq,           // tag
        0x00000060u32, // payload size
        0x00000000u32, // command = status
        0x00000015u32, // address
        0x00000000u32, // unk
    ];
    command(&device, status_cmd);

    // read response
    response(&device);

    // read status
    assert!(status(&device, status_cmd, seq));

    let firmware: Vec<u8> = std::fs::read(filename).expect("Failed to open file");

    const PAGESIZE: usize = 0x4000;
    let pages = firmware.chunks(PAGESIZE);
    let addresses = (0x0008_0000u32..).step_by(PAGESIZE);

    for (address, page) in addresses.zip(pages) {
        seq += 1;

        // write command
        let page_cmd = &[
            0x06022009u32,       // magic
            seq,                 // tag
            (page.len() as u32), // payload size
            0x00000003u32,       // command = status
            address,             // address
            0x00000000u32,       // unk
        ];
        command(&device, page_cmd);

        // write data
        for packet in page.chunks(512) {
            println!(
                "SEQ {} - ADDRESS {:x} - PACKET {}",
                seq,
                address,
                packet.len()
            );
            device
                .write_bulk(KINECT_AUDIO_ENDPOINT_OUT, packet, TIMEOUT)
                .unwrap();
        }

        // read status
        assert!(status(&device, page_cmd, seq));
    }

    seq += 1;

    let finished_cmd = &[
        0x0602_2009u32, // magic
        seq,            // tag
        0x0000_0000u32, // payload size
        0x0000_0004u32, // command = status
        0x0008_0030u32, // address
        0x0000_0000u32, // unk
    ];

    // write command
    command(&device, finished_cmd);

    // read status
    assert!(status(&device, finished_cmd, seq));
}
