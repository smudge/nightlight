extern crate gio;

use gio::SettingsExt;
use std::os::raw::{c_float, c_int};

mod locale;

pub use self::locale::Locale;

pub struct Client {
    settings: gio::Settings,
}

impl Client {
    pub fn new() -> Client {
        Client {
            settings: gio::Settings::new("org.gnome.settings-daemon.plugins.color"),
        }
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        match self.settings.set_boolean("night-light-enabled", enabled) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Failed to set enabled to {}", enabled).to_string()),
        }
    }

    pub fn set_mode(&self, mode: c_int) -> Result<(), String> {
        Ok(())
    }

    pub fn set_schedule(&self, from: (u8, u8), to: (u8, u8)) -> Result<(), String> {
        Ok(())
    }

    pub fn set_strength(&self, strength: c_float) -> Result<(), String> {
        let ratio = strength / 100.0;
        let kelvins = (1.0 - ratio) * (6000.0 - 3500.0) + 3500.0;
        match self
            .settings
            .set_uint("night-light-temperature", kelvins as u32)
        {
            Ok(_) => Ok(()),
            Err(_) => Err("Unable to set temperature".to_string()),
        }
    }

    pub fn get_strength(&self) -> Result<i32, String> {
        let kelvins = self.settings.get_uint("night-light-temperature");
        let ratio = 1.0 - (kelvins as f64 - 3500.0) / (6000.0 - 3500.0);
        Ok((ratio * 100.0) as i32)
    }

    pub fn get_enabled(&self) -> Result<bool, String> {
        Ok(self.settings.get_boolean("night-light-enabled"))
    }

    pub fn get_mode(&self) -> Result<i32, String> {
        Ok(0)
    }

    pub fn get_schedule(&self) -> Result<((u8, u8), (u8, u8)), String> {
        Ok(((0, 0), (1, 1)))
    }
}
