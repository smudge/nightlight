extern crate objc;
extern crate time;

mod ffi;

use std::fmt;

pub struct NightShift {
    client: ffi::CBBlueLightClient,
}

pub struct Status {
    pub currently_active: bool,
    pub schedule: Schedule,
    pub color_temperature: i32,
}

pub struct Time {
    hour: u8,
    min: u8,
}

impl Time {
    pub fn parse(value: &String) -> Result<Time, String> {
        let mut time = time::Time::parse(value, "%-H:%M");
        if time.is_err() {
            time = time::Time::parse(value, "%-H");
        }
        if time.is_err() {
            time = time::Time::parse(value.to_uppercase(), "%-I:%M%P");
        }
        if time.is_err() {
            time = time::Time::parse(value.to_uppercase(), "%-I:%M%P");
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

impl NightShift {
    pub fn new() -> NightShift {
        NightShift {
            client: ffi::CBBlueLightClient::new(),
        }
    }

    pub fn on(&self) -> Result<(), String> {
        self.toggle(true)
    }

    pub fn off(&self) -> Result<(), String> {
        self.toggle(false)
    }

    pub fn toggle(&self, on: bool) -> Result<(), String> {
        self.client.set_enabled(on)
    }

    pub fn set_schedule(&self, schedule: Schedule) -> Result<(), String> {
        match schedule {
            Schedule::Off => self.client.set_mode(0),
            Schedule::SunsetToSunrise => self.client.set_mode(1),
            Schedule::Custom(from, to) => {
                self.client.set_mode(2)?;
                self.client.set_schedule(from.tuple(), to.tuple())
            }
        }
    }

    pub fn set_temp(&self, temp: i32) -> Result<(), String> {
        if temp < 0 || temp > 100 {
            return Err("Color temperature must be a number from 0 to 100.".to_string());
        }

        self.client.set_strength(temp as f32 / 100.0)
    }

    pub fn status(&self) -> Result<Status, String> {
        let status = self.client.status()?;
        let schedule = NightShift::schedule(status.mode(), status.from_time(), status.to_time())?;
        Ok(Status {
            currently_active: status.enabled(),
            schedule,
            color_temperature: self.client.get_strength()?,
        })
    }

    fn schedule(mode: i32, from: (u8, u8), to: (u8, u8)) -> Result<Schedule, String> {
        let from = Time::from_tuple(from);
        let to = Time::from_tuple(to);

        match mode {
            0 => Ok(Schedule::Off),
            2 => Ok(Schedule::Custom(from, to)),
            1 => Ok(Schedule::SunsetToSunrise),
            _ => Err("Unrecognized schedule type".to_string()),
        }
    }
}
