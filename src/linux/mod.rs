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

// A small seam for testing schedule updates without a live GNOME session.
trait ScheduleSettings {
    fn read_boolean(&self, key: &str) -> bool;
    fn write_boolean(&self, key: &str, value: bool) -> Result<(), String>;
    fn write_double(&self, key: &str, value: f64) -> Result<(), String>;
}

impl ScheduleSettings for gio::Settings {
    fn read_boolean(&self, key: &str) -> bool {
        self.get_boolean(key)
    }

    fn write_boolean(&self, key: &str, value: bool) -> Result<(), String> {
        self.set_boolean(key, value).map_err(|_| format!("Unable to set {}", key))
    }

    fn write_double(&self, key: &str, value: f64) -> Result<(), String> {
        self.set_double(key, value).map_err(|_| format!("Unable to set {}", key))
    }
}

fn set_mode_on<S: ScheduleSettings>(settings: &S, mode: i32) -> Result<(), String> {
    let (enabled, automatic) = mode_settings(mode);
    settings.write_boolean("night-light-enabled", enabled)?;
    settings.write_boolean("night-light-schedule-automatic", automatic)
}

fn get_mode_from<S: ScheduleSettings>(settings: &S) -> i32 {
    mode_from_settings(
        settings.read_boolean("night-light-enabled"),
        settings.read_boolean("night-light-schedule-automatic"),
    )
}

fn set_schedule_on<S: ScheduleSettings>(
    settings: &S, from: (u8, u8), to: (u8, u8),
) -> Result<(), String> {
    let from = from.0 as f64 + (from.1 as f64 / 60.0);
    let to = to.0 as f64 + (to.1 as f64 / 60.0);
    settings.write_double("night-light-schedule-from", from)?;
    settings.write_double("night-light-schedule-to", to)
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
        set_mode_on(&self.settings, mode)?;
        gio::Settings::sync();
        Ok(())
    }

    pub fn set_schedule(&self, from: (u8, u8), to: (u8, u8)) -> Result<(), String> {
        set_schedule_on(&self.settings, from, to)?;
        gio::Settings::sync();
        Ok(())
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
        Ok(get_mode_from(&self.settings))
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
    use super::{mode_from_settings, mode_settings, set_mode_on, get_mode_from, set_schedule_on, ScheduleSettings};
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    struct FakeSettings {
        booleans: RefCell<HashMap<String, bool>>,
        doubles: RefCell<HashMap<String, f64>>,
        rejected_key: Option<&'static str>,
    }

    impl ScheduleSettings for FakeSettings {
        fn read_boolean(&self, key: &str) -> bool {
            *self.booleans.borrow().get(key).unwrap_or(&false)
        }

        fn write_boolean(&self, key: &str, value: bool) -> Result<(), String> {
            match self.rejected_key {
                Some(rejected) if rejected == key => Err(format!("Unable to set {}", key)),
                _ => {
                    self.booleans.borrow_mut().insert(key.to_string(), value);
                    Ok(())
                }
            }
        }

        fn write_double(&self, key: &str, value: f64) -> Result<(), String> {
            match self.rejected_key {
                Some(rejected) if rejected == key => Err(format!("Unable to set {}", key)),
                _ => {
                    self.doubles.borrow_mut().insert(key.to_string(), value);
                    Ok(())
                }
            }
        }
    }

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
    #[test]
    fn fake_settings_exercise_real_mode_transitions() {
        let settings = FakeSettings::default();
        for mode in [1, 2, 0, 2, 1] {
            set_mode_on(&settings, mode).unwrap();
            assert_eq!(get_mode_from(&settings), mode);
        }
    }

    #[test]
    fn fake_settings_store_fractional_schedule_hours() {
        let settings = FakeSettings::default();
        set_schedule_on(&settings, (7, 30), (21, 15)).unwrap();
        let values = settings.doubles.borrow();
        assert_eq!(values["night-light-schedule-from"], 7.5);
        assert_eq!(values["night-light-schedule-to"], 21.25);
    }

    #[test]
    fn fake_settings_propagate_write_failure() {
        let settings = FakeSettings {
            rejected_key: Some("night-light-schedule-automatic"),
            ..Default::default()
        };
        assert!(set_mode_on(&settings, 1).is_err());

        let settings = FakeSettings {
            rejected_key: Some("night-light-schedule-to"),
            ..Default::default()
        };
        assert!(set_schedule_on(&settings, (8, 0), (20, 0)).is_err());
    }

}
