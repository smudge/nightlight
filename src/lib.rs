extern crate objc;

mod ffi;
mod schedule;
mod status;

pub use schedule::{Schedule, Time};
pub use status::Status;

pub struct NightLight {
    client: ffi::CBBlueLightClient,
}

impl NightLight {
    pub fn new() -> NightLight {
        NightLight {
            client: ffi::CBBlueLightClient::new(),
        }
    }

    pub fn on(&self) -> Result<(), String> {
        self.toggle(Status::On)
    }

    pub fn off(&self) -> Result<(), String> {
        self.toggle(Status::Off)
    }

    pub fn toggle(&self, status: Status) -> Result<(), String> {
        match self.client.set_enabled(status.as_bool()) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Failed to turn Night Shift {}", status).to_string()),
        }
    }

    pub fn set_schedule(&self, schedule: Schedule) -> Result<(), String> {
        let status = self.status()?;

        match schedule {
            Schedule::Off => self.client.set_mode(0)?,
            Schedule::SunsetToSunrise => self.client.set_mode(1)?,
            Schedule::Custom(from, to) => {
                self.client.set_mode(2)?;
                self.client.set_schedule(from.tuple(), to.tuple())?
            }
        }
        self.toggle(status)
    }

    pub fn get_schedule(&self) -> Result<Schedule, String> {
        let status = self.client.status()?;
        NightLight::schedule(status.mode(), status.from_time(), status.to_time())
    }

    pub fn set_temp(&self, temp: i32) -> Result<(), String> {
        if temp < 0 || temp > 100 {
            return Err("Color temperature must be a number from 0 to 100.".to_string());
        }

        self.client.set_strength(temp as f32 / 100.0)
    }

    pub fn get_temp(&self) -> Result<i32, String> {
        self.client.get_strength()
    }

    pub fn status(&self) -> Result<Status, String> {
        Ok(match self.client.status()?.enabled() {
            true => Status::On,
            false => Status::Off,
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
