extern crate objc;

mod ffi;

use std::fmt;

pub struct NightShift {
    client: ffi::Client,
}

pub struct Status {
    pub currently_active: bool,
    pub schedule_type: Schedule,
    pub color_temperature: i32,
    pub from_time: String,
    pub to_time: String,
}

pub enum Schedule {
    Off = 0,
    Custom = 2,
    SunsetToSunrise = 1,
}

impl fmt::Display for Schedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Schedule::Off => write!(f, "off"),
            Schedule::Custom => write!(f, "custom"),
            Schedule::SunsetToSunrise => write!(f, "sunset to sunrise"),
        }
    }
}

impl NightShift {
    pub fn new() -> NightShift {
        NightShift {
            client: ffi::Client::new(),
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
        Ok(Status {
            currently_active: status.enabled(),
            schedule_type: NightShift::schedule_type(status.mode())?,
            color_temperature: self.client.get_strength()?,
            from_time: status.from_time(),
            to_time: status.to_time(),
        })
    }

    pub fn schedule_type(mode: i32) -> Result<Schedule, String> {
        match mode {
            0 => Ok(Schedule::Off),
            2 => Ok(Schedule::Custom),
            1 => Ok(Schedule::SunsetToSunrise),
            _ => Err("Unrecognized schedule type".to_string()),
        }
    }
}
