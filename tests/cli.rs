use std::process::Command;

#[test]
fn invalid_firmware_exits_with_readable_errors() {
    let path = std::env::temp_dir().join(format!("kinect-invalid-firmware-{}", std::process::id()));
    let check = |message: &str| {
        let output = Command::new(env!("CARGO_BIN_EXE_kinect-firmware-utils"))
            .arg(&path)
            .output()
            .expect("run uploader");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(1), "{stderr}");
        assert!(stderr.contains(message), "{stderr}");
        assert!(!stderr.contains("panicked"), "{stderr}");
    };

    check(&format!("could not read firmware '{}'", path.display()));

    std::fs::write(&path, [0; 24]).expect("write bad magic");
    check("could not parse firmware header");

    std::fs::write(&path, b"\x0d\xf0\x77\xca").expect("write truncated header");
    check("could not parse firmware header");

    let mut data = [0; 24];
    data[..4].copy_from_slice(b"\x0d\xf0\x77\xca");
    for expected in [16u32, 32] {
        data[16..20].copy_from_slice(&expected.to_le_bytes());
        std::fs::write(&path, data).expect("write mismatched size");
        check(&format!(
            "firmware size mismatch: header declares {expected} bytes, file contains 24"
        ));
    }

    std::fs::remove_file(&path).expect("remove synthetic firmware");
}
