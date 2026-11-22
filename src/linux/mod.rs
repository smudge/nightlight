extern crate gio;

use crate::backend::Backend;
use crate::{Schedule, Status};
use gio::SettingsExt;
use crate::schedule::Times;
use std::os::raw::c_int;

mod locale;

pub use self::locale::Locale;

pub(crate) fn backend() -> Box<dyn Backend> {
    Box::new(Gnome {
        client: Client::new(),
    })
}

struct Gnome {
    client: Client,
}

impl Backend for Gnome {
    fn name(&self) -> &'static str {
        "gnome"
    }

    fn status(&self) -> Result<Status, String> {
        Ok(Status::from_bool(self.client.get_enabled()?))
    }

    fn set_status(&self, status: Status) -> Result<(), String> {
        self.client.set_enabled(status.as_bool())
    }

    fn schedule(&self) -> Result<Schedule, String> {
        Schedule::from_mode(self.client.get_mode()?, || self.client.get_schedule())
    }

    fn set_schedule(&self, schedule: Schedule) -> Result<(), String> {
        match schedule {
            Schedule::Off => self.client.set_schedule_mode(0, None),
            Schedule::SunsetToSunrise => self.client.set_schedule_mode(1, None),
            Schedule::Custom(from, to) => {
                self.client.set_schedule_mode(2, Some((from.tuple(), to.tuple())))
            }
        }
    }

    fn strength(&self) -> Result<i32, String> {
        self.client.get_strength()
    }

    fn set_strength(&self, strength: i32) -> Result<(), String> {
        self.client.set_strength(strength as f32 / 100.0)
    }
}

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
    fn begin(&self);
    fn commit(&self);
    fn rollback(&self);
    fn read_boolean(&self, key: &str) -> bool;
    fn write_boolean(&self, key: &str, value: bool) -> Result<(), String>;
    fn write_double(&self, key: &str, value: f64) -> Result<(), String>;
}

impl ScheduleSettings for gio::Settings {
    fn begin(&self) { self.delay(); }
    fn commit(&self) { self.apply(); }
    fn rollback(&self) { self.revert(); }
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

// GSettings holds staged changes locally until apply; revert discards writes
// if any key update fails. Schedule configuration is committed as one group.
fn update_schedule_on<S: ScheduleSettings>(
    settings: &S,
    mode: i32,
    times: Option<Times>,
) -> Result<(), String> {
    settings.begin();
    let result = match times {
        Some((from, to)) => set_schedule_on(settings, from, to)
            .and_then(|()| set_mode_on(settings, mode)),
        None => set_mode_on(settings, mode),
    };
    match result {
        Ok(()) => { settings.commit(); Ok(()) }
        Err(error) => { settings.rollback(); Err(error) }
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
        self.settings
            .set_boolean("night-light-enabled", enabled)
            .map_err(|_| format!("Failed to set enabled to {}", enabled))?;
        gio::Settings::sync();
        Ok(())
    }

    pub fn set_schedule_mode(
        &self,
        mode: c_int,
        times: Option<Times>,
    ) -> Result<(), String> {
        update_schedule_on(&self.settings, mode, times)?;
        gio::Settings::sync();
        Ok(())
    }

    pub fn set_strength(&self, ratio: f32) -> Result<(), String> {
        let kelvins = (1.0 - ratio) * (6000.0 - 3500.0) + 3500.0;
        self.settings
            .set_uint("night-light-temperature", kelvins as u32)
            .map_err(|_| "Unable to set temperature".to_string())?;
        gio::Settings::sync();
        Ok(())
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

    pub fn get_schedule(&self) -> Result<Times, String> {
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
    use super::{mode_from_settings, mode_settings, set_mode_on, get_mode_from, set_schedule_on, update_schedule_on, ScheduleSettings};
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    struct FakeSettings {
        booleans: RefCell<HashMap<String, bool>>,
        doubles: RefCell<HashMap<String, f64>>,
        rejected_key: Option<&'static str>,
        pending_bools: RefCell<Option<HashMap<String, bool>>>,
        pending_doubles: RefCell<Option<HashMap<String, f64>>>,
    }

    impl ScheduleSettings for FakeSettings {
        fn begin(&self) {
            *self.pending_bools.borrow_mut() = Some(HashMap::new());
            *self.pending_doubles.borrow_mut() = Some(HashMap::new());
        }

        fn commit(&self) {
            if let Some(pending) = self.pending_bools.borrow_mut().take() {
                self.booleans.borrow_mut().extend(pending);
            }
            if let Some(pending) = self.pending_doubles.borrow_mut().take() {
                self.doubles.borrow_mut().extend(pending);
            }
        }

        fn rollback(&self) {
            self.pending_bools.borrow_mut().take();
            self.pending_doubles.borrow_mut().take();
        }

        fn read_boolean(&self, key: &str) -> bool {
            *self.booleans.borrow().get(key).unwrap_or(&false)
        }

        fn write_boolean(&self, key: &str, value: bool) -> Result<(), String> {
            match self.rejected_key {
                Some(rejected) if rejected == key => Err(format!("Unable to set {}", key)),
                _ => {
                    match self.pending_bools.borrow_mut().as_mut() {
                        Some(pending) => { pending.insert(key.to_string(), value); }
                        None => { self.booleans.borrow_mut().insert(key.to_string(), value); }
                    }
                    Ok(())
                }
            }
        }

        fn write_double(&self, key: &str, value: f64) -> Result<(), String> {
            match self.rejected_key {
                Some(rejected) if rejected == key => Err(format!("Unable to set {}", key)),
                _ => {
                    match self.pending_doubles.borrow_mut().as_mut() {
                        Some(pending) => { pending.insert(key.to_string(), value); }
                        None => { self.doubles.borrow_mut().insert(key.to_string(), value); }
                    }
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

    #[test]
    fn entire_custom_schedule_commits_together() {
        let settings = FakeSettings::default();
        update_schedule_on(&settings, 2, Some(((7, 30), (21, 15)))).unwrap();
        assert_eq!(get_mode_from(&settings), 2);
        let times = settings.doubles.borrow();
        assert_eq!(times["night-light-schedule-from"], 7.5);
        assert_eq!(times["night-light-schedule-to"], 21.25);
    }

    #[test]
    fn failed_mode_write_discards_staged_times() {
        let settings = FakeSettings {
            rejected_key: Some("night-light-schedule-automatic"),
            ..Default::default()
        };
        assert!(update_schedule_on(&settings, 2, Some(((7, 30), (21, 15)))).is_err());
        assert_eq!(get_mode_from(&settings), 0);
        assert!(settings.doubles.borrow().is_empty());
        assert!(settings.pending_doubles.borrow().is_none());
    }

    #[test]
    fn failed_time_write_discards_prior_staged_values() {
        let settings = FakeSettings {
            rejected_key: Some("night-light-schedule-to"),
            ..Default::default()
        };
        assert!(update_schedule_on(&settings, 2, Some(((7, 30), (21, 15)))).is_err());
        assert!(settings.doubles.borrow().is_empty());
        assert!(settings.booleans.borrow().is_empty());
    }

    #[test]
    fn mode_only_update_commits_and_preserves_times() {
        let settings = FakeSettings::default();
        update_schedule_on(&settings, 2, Some(((7, 30), (21, 15)))).unwrap();
        update_schedule_on(&settings, 0, None).unwrap();
        assert_eq!(get_mode_from(&settings), 0);
        assert_eq!(settings.doubles.borrow()["night-light-schedule-from"], 7.5);
    }

}
