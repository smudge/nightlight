#[cfg_attr(target_os = "linux", path = "linux/mod.rs")]
#[cfg_attr(target_os = "macos", path = "macos/mod.rs")]
mod os;
mod backend;
mod schedule;
mod status;

use backend::Backend;

pub use schedule::{Schedule, Time};
pub use status::Status;

pub struct NightLight {
    backend: Box<dyn Backend>,
}

impl Default for NightLight {
    fn default() -> NightLight {
        NightLight::new()
    }
}

impl NightLight {
    pub fn new() -> NightLight {
        NightLight {
            backend: os::backend(),
        }
    }

    /// The backend in use, e.g. "macos" or "gnome".
    pub fn backend_name(&self) -> &'static str {
        self.backend.name()
    }

    pub fn on(&self) -> Result<(), String> {
        self.toggle(Status::On)
    }

    pub fn off(&self) -> Result<(), String> {
        self.toggle(Status::Off)
    }

    pub fn toggle(&self, status: Status) -> Result<(), String> {
        match self.backend.set_status(status) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Failed to turn Night Shift {}", status).to_string()),
        }
    }

    pub fn set_schedule(&self, schedule: Schedule) -> Result<(), String> {
        self.backend.set_schedule(schedule)
    }

    pub fn get_schedule(&self) -> Result<Schedule, String> {
        self.backend.schedule()
    }

    pub fn set_temp(&self, temp: i32) -> Result<(), String> {
        if !(0..=100).contains(&temp) {
            return Err("Color temperature must be a number from 0 to 100.".to_string());
        }

        self.backend.set_strength(temp)
    }

    pub fn get_temp(&self) -> Result<i32, String> {
        self.backend.strength()
    }

    pub fn status(&self) -> Result<Status, String> {
        self.backend.status()
    }
}
