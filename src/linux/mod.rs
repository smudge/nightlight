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
            Ok(_) => Ok(gio::Settings::sync()),
            Err(_) => Err(format!("Failed to set enabled to {}", enabled).to_string()),
        }
    }

    pub fn set_mode(&self, mode: c_int) -> Result<(), String> {
        let (enabled, scheduled) = match mode {
            1 => (true, true),
            2 => (true, false),
            _ => (false, false),
        };

        self.set_enabled(enabled)?;
        match self
            .settings
            .set_boolean("night-light-schedule-automatic", scheduled)
        {
            Ok(_) => Ok(gio::Settings::sync()),
            Err(_) => Err("Unable to set schedule!".to_string()),
        }
    }

    pub fn set_schedule(&self, from: (u8, u8), to: (u8, u8)) -> Result<(), String> {
        let from = from.0 as f64 + (from.1 as f64 / 60.0);
        let to = to.0 as f64 + (to.1 as f64 / 60.0);
        match self.settings.set_double("night-light-schedule-from", from) {
            Ok(_) => match self.settings.set_double("night-light-schedule-to", to) {
                Ok(_) => Ok(gio::Settings::sync()),
                Err(_) => Err("Unable to set schedule!".to_string()),
            },
            Err(_) => Err("Unable to set schedule!".to_string()),
        }
    }

    pub fn set_strength(&self, ratio: f32) -> Result<(), String> {
        let kelvins = (1.0 - ratio) * (6000.0 - 3500.0) + 3500.0;
        match self
            .settings
            .set_uint("night-light-temperature", kelvins as u32)
        {
            Ok(_) => Ok(gio::Settings::sync()),
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
        match (
            self.get_enabled()?,
            self.settings.get_boolean("night-light-schedule-automatic"),
        ) {
            (true, true) => Ok(1),
            (true, false) => Ok(2),
            (false, _) => Ok(0),
        }
    }

    pub fn get_schedule(&self) -> Result<((u8, u8), (u8, u8)), String> {
        let from = self.settings.get_double("night-light-schedule-from");
        let to = self.settings.get_double("night-light-schedule-to");
        Ok((
            (from.trunc() as u8, (from.fract() * 60.0).round() as u8),
            (to.trunc() as u8, (to.fract() * 60.0).round() as u8),
        ))
    }
}
