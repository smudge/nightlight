extern crate time;

use crate::os::Locale;
use std::fmt;

pub struct Time {
    inner: time::Time,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Time {
    fn new(inner: time::Time) -> Result<Time, String> {
        Locale::initialize()?;
        Ok(Time { inner })
    }

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

        match time {
            Ok(time) => Time::new(time),
            Err(_) => Err(format!("Invalid string value '{}'", value)),
        }
    }

    pub fn from_tuple(tuple: (u8, u8)) -> Result<Time, String> {
        match time::Time::try_from_hms(tuple.0, tuple.1, 0) {
            Ok(time) => Time::new(time),
            Err(_) => Err("Unable to read time value".to_string()),
        }
    }

    pub fn tuple(&self) -> (u8, u8) {
        (self.inner.hour(), self.inner.minute())
    }

    pub fn to_string(&self) -> String {
        if Locale::current().is_24_hr() {
            self.inner.format("%-H:%M")
        } else {
            self.inner.format("%-I:%M%p")
        }
    }
}
