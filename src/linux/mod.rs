extern crate gio;

use gio::SettingsExt;
use std::os::raw::{c_float, c_int};

mod locale;

pub use self::locale::Locale;

fn mode_settings(mode: i32) -> (bool, bool) {
    match mode {
        1 => (true, true),
        2 => (true, false),
        _ => (false, false),
    }
}

fn mode_from_settings(enabled: bool, automatic: bool) -> i32 {
    match (enabled, automatic) {
        (true, true) => 1,
        (true, false) => 2,
        (false, _) => 0,
    }
}

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
        let (enabled, scheduled) = mode_settings(mode);

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
        Ok(mode_from_settings(
            self.get_enabled()?,
            self.settings.get_boolean("night-light-schedule-automatic"),
        ))
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

#[cfg(test)]
mod tests {
    use super::{mode_from_settings, mode_settings};

    #[test]
    fn every_schedule_mode_round_trips() {
        for mode in [0, 1, 2] {
            let (enabled, automatic) = mode_settings(mode);
            assert_eq!(mode_from_settings(enabled, automatic), mode);
        }
    }

    #[test]
    fn disabled_night_light_is_not_reported_as_custom() {
        assert_eq!(mode_from_settings(false, false), 0);
        assert_eq!(mode_from_settings(false, true), 0);
        assert_eq!(mode_from_settings(true, false), 2);
        assert_eq!(mode_from_settings(true, true), 1);
    }

    #[test]
    fn stopping_schedule_disables_night_light() {
        assert_eq!(mode_settings(0), (false, false));
    }
}
