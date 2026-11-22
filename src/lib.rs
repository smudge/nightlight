#[cfg_attr(target_os = "linux", path = "linux/mod.rs")]
#[cfg_attr(target_os = "macos", path = "macos/mod.rs")]
mod os;
mod schedule;
mod status;

pub use schedule::{Schedule, Time};
pub use status::Status;

pub struct NightLight {
    client: os::Client,
}

impl Default for NightLight {
    fn default() -> NightLight {
        NightLight::new()
    }
}

impl NightLight {
    pub fn new() -> NightLight {
        NightLight {
            client: os::Client::new(),
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
        // macOS allows a schedule to change independently of the current on/off state.
        // On GNOME, "night-light-enabled" is part of the schedule configuration.
        #[cfg(target_os = "macos")]
        let status = self.status()?;

        #[cfg(target_os = "macos")]
        match schedule {
            Schedule::Off => self.client.set_mode(0)?,
            Schedule::SunsetToSunrise => self.client.set_mode(1)?,
            Schedule::Custom(from, to) => {
                self.client.set_mode(2)?;
                self.client.set_schedule(from.tuple(), to.tuple())?
            }
        }

        #[cfg(target_os = "linux")]
        match schedule {
            Schedule::Off => self.client.set_schedule_mode(0, None)?,
            Schedule::SunsetToSunrise => self.client.set_schedule_mode(1, None)?,
            Schedule::Custom(from, to) => {
                self.client.set_schedule_mode(2, Some((from.tuple(), to.tuple())))?
            }
        }

        #[cfg(target_os = "macos")]
        {
            self.toggle(status)
        }
        #[cfg(target_os = "linux")]
        {
            Ok(())
        }
    }

    pub fn get_schedule(&self) -> Result<Schedule, String> {
        NightLight::schedule(self.client.get_mode()?, || self.client.get_schedule())
    }

    pub fn set_temp(&self, temp: i32) -> Result<(), String> {
        if !(0..=100).contains(&temp) {
            return Err("Color temperature must be a number from 0 to 100.".to_string());
        }

        self.client.set_strength(temp as f32 / 100.0)
    }

    pub fn get_temp(&self) -> Result<i32, String> {
        self.client.get_strength()
    }

    pub fn status(&self) -> Result<Status, String> {
        Ok(match self.client.get_enabled()? {
            true => Status::On,
            false => Status::Off,
        })
    }

    fn schedule<F>(mode: i32, read_times: F) -> Result<Schedule, String>
    where
        F: FnOnce() -> Result<schedule::Times, String>,
    {
        match mode {
            0 => Ok(Schedule::Off),
            1 => Ok(Schedule::SunsetToSunrise),
            2 => {
                let (from, to) = read_times()?;
                Ok(Schedule::Custom(Time::from_tuple(from)?, Time::from_tuple(to)?))
            }
            _ => Err("Unrecognized schedule type".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NightLight, Schedule};

    #[test]
    fn non_custom_schedules_ignore_invalid_stored_times() {
        assert!(matches!(
            NightLight::schedule(0, || panic!("should not read unused schedule times")),
            Ok(Schedule::Off)
        ));
        assert!(matches!(
            NightLight::schedule(1, || panic!("should not read unused schedule times")),
            Ok(Schedule::SunsetToSunrise)
        ));
    }

    #[test]
    fn custom_schedule_requires_valid_times() {
        let from = (8, 30);
        let to = (22, 15);

        match NightLight::schedule(2, || Ok((from, to))).unwrap() {
            Schedule::Custom(start, end) => {
                assert_eq!(start.tuple(), from);
                assert_eq!(end.tuple(), to);
            }
            _ => panic!("Expected custom schedule"),
        }

        assert!(NightLight::schedule(2, || Ok(((24, 0), to))).is_err());
        assert!(NightLight::schedule(2, || Ok((from, (23, 60)))).is_err());
    }

    #[test]
    fn custom_schedule_propagates_backend_read_errors() {
        assert!(matches!(
            NightLight::schedule(2, || Err("backend read failed".into())),
            Err(message) if message == "backend read failed"
        ));
    }

    #[test]
    fn unknown_schedule_mode_has_its_own_error() {
        assert!(matches!(
            NightLight::schedule(99, || panic!("should not read unknown mode")),
            Err(message) if message == "Unrecognized schedule type"
        ));
    }
}
