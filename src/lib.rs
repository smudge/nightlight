extern crate objc;

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
    pub display: String,
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

    fn schedule(mode: i32, from_time: String, to_time: String) -> Result<Schedule, String> {
        let from_time = Time { display: from_time };
        let to_time = Time { display: to_time };

        match mode {
            0 => Ok(Schedule::Off),
            2 => Ok(Schedule::Custom(from_time, to_time)),
            1 => Ok(Schedule::SunsetToSunrise),
            _ => Err("Unrecognized schedule type".to_string()),
        }
    }
}
