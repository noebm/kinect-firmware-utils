/// A validated tilt angle in whole degrees from -31 to 31.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tilt(i32);

impl Tilt {
    /// Returns `None` for angles outside -31..=31 degrees.
    pub fn new(degrees: i32) -> Option<Self> {
        if !(-31..=31).contains(&degrees) {
            return None;
        }

        Some(Self(degrees))
    }

    pub fn degrees(self) -> i32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_angles_in_safe_range() {
        for degrees in -31..=31 {
            assert_eq!(Tilt::new(degrees).expect("valid angle").degrees(), degrees);
        }
    }

    #[test]
    fn rejects_angles_outside_safe_range() {
        for degrees in [i32::MIN, -32, 32, i32::MAX] {
            assert_eq!(Tilt::new(degrees), None);
        }
    }
}
