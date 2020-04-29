extern crate objc;

mod ffi;
mod schedule;

pub use schedule::{Schedule, Time};

pub struct NightLight {
    client: ffi::CBBlueLightClient,
}

pub struct Status {
    pub currently_active: bool,
    pub schedule: Schedule,
    pub color_temperature: i32,
}

impl NightLight {
    pub fn new() -> NightLight {
        NightLight {
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
        let was_on = self.status()?.currently_active;

        match schedule {
            Schedule::Off => self.client.set_mode(0)?,
            Schedule::SunsetToSunrise => self.client.set_mode(1)?,
            Schedule::Custom(from, to) => {
                self.client.set_mode(2)?;
                self.client.set_schedule(from.tuple(), to.tuple())?
            }
        }
        self.toggle(was_on)
    }

    pub fn set_temp(&self, temp: i32) -> Result<(), String> {
        if temp < 0 || temp > 100 {
            return Err("Color temperature must be a number from 0 to 100.".to_string());
        }

        self.client.set_strength(temp as f32 / 100.0)
    }

    pub fn status(&self) -> Result<Status, String> {
        let status = self.client.status()?;
        let schedule = NightLight::schedule(status.mode(), status.from_time(), status.to_time())?;
        Ok(Status {
            currently_active: status.enabled(),
            schedule,
            color_temperature: self.client.get_strength()?,
        })
    }

    fn schedule(mode: i32, from: (u8, u8), to: (u8, u8)) -> Result<Schedule, String> {
        let from = Time::from_tuple(from)?;
        let to = Time::from_tuple(to)?;

        match mode {
            0 => Ok(Schedule::Off),
            2 => Ok(Schedule::Custom(from, to)),
            1 => Ok(Schedule::SunsetToSunrise),
            _ => Err("Unrecognized schedule type".to_string()),
        }
    }
}
