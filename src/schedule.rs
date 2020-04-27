extern crate time;

use std::fmt;

pub struct Time {
    hour: u8,
    min: u8,
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
            Ok(time) => Ok(Time {
                hour: time.hour(),
                min: time.minute(),
            }),
            Err(_) => Err(format!("Invalid string value '{}'", value)),
        }
    }

    pub fn from_tuple(tuple: (u8, u8)) -> Time {
        Time {
            hour: tuple.0,
            min: tuple.1,
        }
    }

    pub fn tuple(&self) -> (u8, u8) {
        (self.hour, self.min)
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.hour, self.min)
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
