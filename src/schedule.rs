extern crate time;

use std::fmt;

pub struct Time {
    inner: time::Time,
}

impl Time {
    pub fn parse(value: &String) -> Result<Time, String> {
        let mut time = time::Time::parse(value.to_uppercase(), "%-I:%M%P");
        if time.is_err() {
            time = time::Time::parse(value.to_uppercase(), "%-I%P");
        }
        if time.is_err() {
            time = time::Time::parse(value, "%-H:%M");
        }
        if time.is_err() {
            time = time::Time::parse(value, "%-H");
        }
        // TODO: Look at system locale and decide if am/pm can be inferred.

        match time {
            Ok(time) => Ok(Time { inner: time }),
            Err(_) => Err(format!("Invalid string value '{}'", value)),
        }
    }

    pub fn from_tuple(tuple: (u8, u8)) -> Result<Time, String> {
        match time::Time::try_from_hms(tuple.0, tuple.1, 0) {
            Ok(time) => Ok(Time { inner: time }),
            Err(_) => Err("Unable to read time value".to_string()),
        }
    }

    pub fn tuple(&self) -> (u8, u8) {
        (self.inner.hour(), self.inner.minute())
    }

    pub fn to_string(&self) -> String {
        self.inner.format("%-I:%M%P")
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub enum Schedule {
    Off,
    Custom(Time, Time),
    SunsetToSunrise,
}

impl fmt::Display for Schedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Schedule::Off => write!(f, "off"),
            Schedule::Custom(_, _) => write!(f, "custom"),
            Schedule::SunsetToSunrise => write!(f, "sunset to sunrise"),
        }
    }
}
