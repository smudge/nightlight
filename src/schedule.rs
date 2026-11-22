mod time;

pub use self::time::Time;
use std::fmt;

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
