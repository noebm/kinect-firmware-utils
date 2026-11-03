use binrw::BinRead;

use super::Error;

/// Accelerometer and motor readings from a Kinect for Windows / model 1473.
/// Accelerometer values are raw signed readings; their units are unspecified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead)]
#[br(little)]
pub struct MotionState {
    pub accelerometer_x: i32,
    pub accelerometer_y: i32,
    pub accelerometer_z: i32,
    /// Current motor angle in whole degrees.
    pub tilt_degrees: i32,
}

impl MotionState {
    /// Parses the 104-byte motion response, including its four-word prefix.
    pub(super) fn parse(payload: &[u8]) -> Result<Self, Error> {
        if payload.len() != 104 {
            return Err(Error::Payload);
        }

        Self::read(&mut std::io::Cursor::new(&payload[16..32])).map_err(|_| Error::Payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_motion_state_response() {
        for degrees in [-31i32, -1, 0, 1, 31] {
            let mut payload = [0xff; 104];
            payload[16..20].copy_from_slice(&(-123456i32).to_le_bytes());
            payload[20..24].copy_from_slice(&789012i32.to_le_bytes());
            payload[24..28].copy_from_slice(&i32::MIN.to_le_bytes());
            payload[28..32].copy_from_slice(&degrees.to_le_bytes());

            assert_eq!(
                MotionState::parse(&payload).expect("valid motion state"),
                MotionState {
                    accelerometer_x: -123456,
                    accelerometer_y: 789012,
                    accelerometer_z: i32::MIN,
                    tilt_degrees: degrees,
                }
            );
        }
    }

    #[test]
    fn reject_invalid_motion_state_response_lengths() {
        for len in [0, 28, 31, 32, 103, 105] {
            assert!(matches!(
                MotionState::parse(&vec![0; len]),
                Err(Error::Payload)
            ));
        }
    }
}
