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

    /// Uses the named backend instead of detecting one.
    pub fn with_backend(name: &str) -> Result<NightLight, String> {
        Ok(NightLight {
            backend: os::backend_named(name)?,
        })
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
        self.backend.set_status(status)
    }

    pub fn set_schedule(&self, schedule: Schedule) -> Result<(), String> {
        self.backend.set_schedule(schedule)
    }

    pub fn get_schedule(&self) -> Result<Schedule, String> {
        self.backend.schedule()
    }

    /// Like `get_schedule`, but `None` for backends without one (e.g. hyprsunset).
    pub fn current_schedule(&self) -> Result<Option<Schedule>, String> {
        match self.backend.has_schedule() {
            true => self.backend.schedule().map(Some),
            false => Ok(None),
        }
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

#[cfg(test)]
mod tests {
    use super::{Backend, NightLight, Schedule, Status, Time};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    struct FakeBackend {
        status: Cell<Status>,
        schedule: Cell<Option<Schedule>>,
        strength: Cell<i32>,
        failure: Option<&'static str>,
        calls: RefCell<Vec<&'static str>>,
    }

    impl FakeBackend {
        fn new() -> FakeBackend {
            FakeBackend {
                status: Cell::new(Status::Off),
                schedule: Cell::new(Some(Schedule::Off)),
                strength: Cell::new(50),
                failure: None,
                calls: RefCell::new(vec![]),
            }
        }

        fn call(&self, name: &'static str) -> Result<(), String> {
            self.calls.borrow_mut().push(name);
            self.failure.map_or(Ok(()), |error| Err(error.to_string()))
        }
    }

    impl Backend for Rc<FakeBackend> {
        fn name(&self) -> &'static str {
            "fake"
        }

        fn status(&self) -> Result<Status, String> {
            self.call("status")?;
            Ok(self.status.get())
        }

        fn set_status(&self, status: Status) -> Result<(), String> {
            self.call("set_status")?;
            self.status.set(status);
            Ok(())
        }

        fn has_schedule(&self) -> bool {
            self.schedule.get().is_some()
        }

        fn schedule(&self) -> Result<Schedule, String> {
            self.call("schedule")?;
            self.schedule.get().ok_or_else(|| "no schedule".to_string())
        }

        fn set_schedule(&self, schedule: Schedule) -> Result<(), String> {
            self.call("set_schedule")?;
            self.schedule.set(Some(schedule));
            Ok(())
        }

        fn strength(&self) -> Result<i32, String> {
            self.call("strength")?;
            Ok(self.strength.get())
        }

        fn set_strength(&self, strength: i32) -> Result<(), String> {
            self.call("set_strength")?;
            self.strength.set(strength);
            Ok(())
        }
    }

    fn night_light(backend: FakeBackend) -> (NightLight, Rc<FakeBackend>) {
        let backend = Rc::new(backend);
        (NightLight { backend: Box::new(backend.clone()) }, backend)
    }

    #[test]
    fn on_and_off_set_the_status() {
        let (night_light, backend) = night_light(FakeBackend::new());
        night_light.on().unwrap();
        assert_eq!(night_light.status(), Ok(Status::On));
        night_light.off().unwrap();
        assert_eq!(backend.status.get(), Status::Off);
        assert_eq!(night_light.backend_name(), "fake");
    }

    #[test]
    fn toggle_failures_come_from_the_backend() {
        let (night_light, _) = night_light(FakeBackend {
            failure: Some("backend unavailable"),
            ..FakeBackend::new()
        });
        assert_eq!(night_light.on(), Err("backend unavailable".to_string()));
    }

    #[test]
    fn schedules_pass_through() {
        let (night_light, _) = night_light(FakeBackend::new());
        let schedule = Schedule::Custom(
            Time::from_tuple((20, 0)).unwrap(),
            Time::from_tuple((6, 0)).unwrap(),
        );
        night_light.set_schedule(schedule).unwrap();
        assert_eq!(night_light.get_schedule(), Ok(schedule));
        assert_eq!(night_light.current_schedule(), Ok(Some(schedule)));
    }

    #[test]
    fn backends_without_schedules_report_none() {
        let (night_light, backend) = night_light(FakeBackend::new());
        backend.schedule.set(None);
        assert_eq!(night_light.current_schedule(), Ok(None));
        assert!(!backend.calls.borrow().contains(&"schedule"));
        assert!(night_light.get_schedule().is_err());
    }

    #[test]
    fn temperature_is_validated_before_reaching_the_backend() {
        let (night_light, backend) = night_light(FakeBackend::new());
        for invalid in [-1, 101] {
            assert!(night_light.set_temp(invalid).is_err());
        }
        assert!(backend.calls.borrow().is_empty());
        night_light.set_temp(70).unwrap();
        assert_eq!(night_light.get_temp(), Ok(70));
    }
}
