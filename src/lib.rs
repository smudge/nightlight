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
        let (from_time, to_time) = self.client.get_schedule()?;
        NightLight::schedule(self.client.get_mode()?, from_time, to_time)
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
        Ok(match self.client.get_enabled()? {
            true => Status::On,
            false => Status::Off,
        })
    }

    fn schedule(mode: i32, from: (u8, u8), to: (u8, u8)) -> Result<Schedule, String> {
        match mode {
            0 => Ok(Schedule::Off),
            1 => Ok(Schedule::SunsetToSunrise),
            2 => Ok(Schedule::Custom(Time::from_tuple(from)?, Time::from_tuple(to)?)),
            _ => Err("Unrecognized schedule type".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NightLight, Schedule};

    #[test]
    fn non_custom_schedules_ignore_invalid_stored_times() {
        let invalid = (255, 255);
        assert!(matches!(
            NightLight::schedule(0, invalid, invalid),
            Ok(Schedule::Off)
        ));
        assert!(matches!(
            NightLight::schedule(1, invalid, invalid),
            Ok(Schedule::SunsetToSunrise)
        ));
    }

    #[test]
    fn custom_schedule_requires_valid_times() {
        let from = (8, 30);
        let to = (22, 15);

        match NightLight::schedule(2, from, to).unwrap() {
            Schedule::Custom(start, end) => {
                assert_eq!(start.tuple(), from);
                assert_eq!(end.tuple(), to);
            }
            _ => panic!("Expected custom schedule"),
        }

        assert!(NightLight::schedule(2, (24, 0), to).is_err());
        assert!(NightLight::schedule(2, from, (23, 60)).is_err());
    }

    #[test]
    fn unknown_schedule_mode_has_its_own_error() {
        assert!(matches!(
            NightLight::schedule(99, (255, 255), (255, 255)),
            Err(message) if message == "Unrecognized schedule type"
        ));
    }
}
