mod time;

pub use self::time::Time;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Schedule {
    Off,
    Custom(Time, Time),
    SunsetToSunrise,
}

impl fmt::Display for Schedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Schedule::Off => write!(f, "off"),
            Schedule::Custom(from, to) => write!(f, "{} to {}", from, to),
            Schedule::SunsetToSunrise => write!(f, "sunset to sunrise"),
        }
    }
}

/// A schedule's (from, to) times as (hour, minute) pairs.
pub(crate) type Times = ((u8, u8), (u8, u8));

impl Schedule {
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub(crate) fn from_mode<F>(mode: i32, read_times: F) -> Result<Schedule, String>
    where
        F: FnOnce() -> Result<Times, String>,
    {
        match mode {
            0 => Ok(Schedule::Off),
            1 => Ok(Schedule::SunsetToSunrise),
            2 => {
                let (from, to) = read_times()?;
                Ok(Schedule::Custom(Time::from_tuple(from)?, Time::from_tuple(to)?))
            }
            _ => Err("Unrecognized schedule type".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Schedule;

    #[test]
    fn non_custom_schedules_ignore_invalid_stored_times() {
        assert_eq!(
            Schedule::from_mode(0, || panic!("should not read unused schedule times")),
            Ok(Schedule::Off)
        );
        assert_eq!(
            Schedule::from_mode(1, || panic!("should not read unused schedule times")),
            Ok(Schedule::SunsetToSunrise)
        );
    }

    #[test]
    fn custom_schedule_requires_valid_times() {
        let from = (8, 30);
        let to = (22, 15);

        match Schedule::from_mode(2, || Ok((from, to))).unwrap() {
            Schedule::Custom(start, end) => {
                assert_eq!(start.tuple(), from);
                assert_eq!(end.tuple(), to);
            }
            _ => panic!("Expected custom schedule"),
        }

        assert!(Schedule::from_mode(2, || Ok(((24, 0), to))).is_err());
        assert!(Schedule::from_mode(2, || Ok((from, (23, 60)))).is_err());
    }

    #[test]
    fn custom_schedule_propagates_backend_read_errors() {
        assert_eq!(
            Schedule::from_mode(2, || Err("backend read failed".into())),
            Err("backend read failed".to_string())
        );
    }

    #[test]
    fn unknown_schedule_mode_has_its_own_error() {
        assert_eq!(
            Schedule::from_mode(99, || panic!("should not read unknown mode")),
            Err("Unrecognized schedule type".to_string())
        );
    }
}
