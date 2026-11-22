use super::command::{Runner, System};
use crate::kelvin;
use super::transaction::{self, Store};
use crate::backend::Backend;
use crate::{Schedule, Status, Time};

const ENABLED: &str = "nightlight.enabled";
const FORCE: &str = "nightlight.force";
const NIGHT: &str = "nightlight.temperature_night";
const DAY: &str = "nightlight.temperature_day";
const CUSTOM: &str = "location.custom_schedule";
const SUNSET: &str = "location.sunset";
const SUNRISE: &str = "location.sunrise";

pub(crate) fn running(runner: &dyn Runner) -> bool {
    runner.run("noctalia", &["msg", "config-get", ENABLED]).is_ok()
}

pub(crate) struct NoctaliaConfig<R: Runner = System> {
    runner: R,
}

impl<R: Runner> Store for NoctaliaConfig<R> {
    fn read(&self, key: &str) -> Result<Option<String>, String> {
        match self.runner.run("noctalia", &["msg", "config-get", key]) {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.contains("path not found") => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn write(&self, key: &str, value: &str) -> Result<(), String> {
        self.runner
            .run("noctalia", &["msg", "config-set", key, value])
            .map(|_| ())
    }
}

pub(crate) struct Noctalia<S: Store = NoctaliaConfig> {
    config: S,
}

impl Noctalia {
    pub fn new() -> Noctalia {
        Noctalia {
            config: NoctaliaConfig { runner: System },
        }
    }
}

fn toml_time(time: Time) -> String {
    let (hour, minute) = time.tuple();
    format!("'{:02}:{:02}'", hour, minute)
}

fn parse_time(key: &str, value: &str) -> Result<Time, String> {
    let text = value.trim_matches(|c| c == '\'' || c == '"');
    text.split_once(':')
        .and_then(|(hour, minute)| Some((hour.parse().ok()?, minute.parse().ok()?)))
        .and_then(|tuple| Time::from_tuple(tuple).ok())
        .ok_or_else(|| format!("Unexpected value for {}: {}", key, value))
}

impl<S: Store> Noctalia<S> {
    fn read<T: std::str::FromStr>(&self, key: &str) -> Result<T, String> {
        let value = self.config.require(key)?;
        value
            .parse()
            .map_err(|_| format!("Unexpected value for {}: {}", key, value))
    }

    fn time(&self, key: &str) -> Result<Time, String> {
        parse_time(key, &self.config.require(key)?)
    }
}

impl<S: Store> Backend for Noctalia<S> {
    fn name(&self) -> &'static str {
        "noctalia"
    }

    fn status(&self) -> Result<Status, String> {
        Ok(Status::from_bool(self.read(ENABLED)? || self.read(FORCE)?))
    }

    fn set_status(&self, status: Status) -> Result<(), String> {
        let changes = match status {
            Status::On => vec![(ENABLED, "true".into())],
            Status::Off => vec![(FORCE, "false".into()), (ENABLED, "false".into())],
        };
        transaction::apply(&self.config, &changes)
    }

    fn schedule(&self) -> Result<Schedule, String> {
        if !self.read::<bool>(ENABLED)? {
            return Ok(Schedule::Off);
        }
        if !self.read::<bool>(CUSTOM)? {
            return Ok(Schedule::SunsetToSunrise);
        }
        Ok(Schedule::Custom(self.time(SUNSET)?, self.time(SUNRISE)?))
    }

    fn set_schedule(&self, schedule: Schedule) -> Result<(), String> {
        let changes = match schedule {
            Schedule::Off => vec![(FORCE, "false".into()), (ENABLED, "false".into())],
            Schedule::SunsetToSunrise => {
                vec![(CUSTOM, "false".into()), (ENABLED, "true".into())]
            }
            Schedule::Custom(from, to) => vec![
                (SUNSET, toml_time(from)),
                (SUNRISE, toml_time(to)),
                (CUSTOM, "true".into()),
                (ENABLED, "true".into()),
            ],
        };
        transaction::apply(&self.config, &changes)
    }

    fn strength(&self) -> Result<i32, String> {
        Ok(kelvin::to_strength(self.read(NIGHT)?))
    }

    fn kelvin(&self) -> Result<Option<u32>, String> {
        self.read(NIGHT).map(Some)
    }

    fn set_strength(&self, strength: i32) -> Result<(), String> {
        let night = kelvin::from_strength(strength);
        let day: u32 = self.read(DAY)?;
        if night + 100 > day {
            return Err(format!(
                "Noctalia needs the night temperature ({}K) to be at least 100K below \
                 its day temperature ({}K)",
                night, day
            ));
        }
        transaction::apply(&self.config, &[(NIGHT, night.to_string())])
    }
}

#[cfg(test)]
mod tests {
    use super::super::transaction::fake::FakeStore;
    use super::*;
    use std::cell::RefCell;

    fn noctalia(values: &[(&str, &str)]) -> Noctalia<FakeStore> {
        Noctalia {
            config: FakeStore::with(values),
        }
    }

    fn defaults() -> Vec<(&'static str, &'static str)> {
        vec![
            (ENABLED, "false"),
            (FORCE, "false"),
            (NIGHT, "4000"),
            (DAY, "6500"),
            (CUSTOM, "false"),
        ]
    }

    fn time(hour: u8, minute: u8) -> Time {
        Time::from_tuple((hour, minute)).unwrap()
    }

    #[test]
    fn every_schedule_round_trips() {
        let noctalia = noctalia(&defaults());
        for schedule in [
            Schedule::SunsetToSunrise,
            Schedule::Custom(time(20, 30), time(6, 0)),
            Schedule::Off,
            Schedule::Custom(time(7, 5), time(21, 45)),
            Schedule::SunsetToSunrise,
        ] {
            noctalia.set_schedule(schedule).unwrap();
            assert_eq!(noctalia.schedule().unwrap(), schedule);
        }
    }

    #[test]
    fn custom_times_are_written_as_toml_strings() {
        let noctalia = noctalia(&defaults());
        noctalia
            .set_schedule(Schedule::Custom(time(20, 30), time(6, 0)))
            .unwrap();
        assert_eq!(noctalia.config.get(SUNSET).unwrap(), "'20:30'");
        assert_eq!(noctalia.config.get(SUNRISE).unwrap(), "'06:00'");
        assert_eq!(noctalia.config.get(CUSTOM).unwrap(), "true");
    }

    #[test]
    fn disabled_night_light_has_no_schedule() {
        let noctalia = noctalia(&[(ENABLED, "false"), (CUSTOM, "true")]);
        assert_eq!(noctalia.schedule().unwrap(), Schedule::Off);
    }

    #[test]
    fn sunset_schedules_do_not_need_custom_times() {
        let noctalia = noctalia(&[(ENABLED, "true"), (CUSTOM, "false")]);
        assert_eq!(noctalia.schedule().unwrap(), Schedule::SunsetToSunrise);
    }

    #[test]
    fn forced_night_light_is_on_and_off_clears_it() {
        let noctalia = noctalia(&[(ENABLED, "false"), (FORCE, "true")]);
        assert_eq!(noctalia.status().unwrap(), Status::On);
        noctalia.set_status(Status::Off).unwrap();
        assert_eq!(noctalia.status().unwrap(), Status::Off);
        assert_eq!(noctalia.config.get(FORCE).unwrap(), "false");
    }

    #[test]
    fn on_enables_the_schedule_without_forcing() {
        let noctalia = noctalia(&defaults());
        noctalia.set_status(Status::On).unwrap();
        assert_eq!(noctalia.status().unwrap(), Status::On);
        assert_eq!(*noctalia.config.writes.borrow(), vec![(ENABLED.to_string(), "true".to_string())]);
    }

    #[test]
    fn failed_enable_leaves_custom_times_unset() {
        let mut noctalia = noctalia(&defaults());
        noctalia.config.rejected_key = Some(ENABLED);
        assert!(noctalia
            .set_schedule(Schedule::Custom(time(20, 30), time(6, 0)))
            .is_err());
        assert_eq!(noctalia.config.get(SUNSET), None);
        assert_eq!(noctalia.config.get(CUSTOM).unwrap(), "false");
    }

    #[test]
    fn strength_maps_onto_the_night_temperature() {
        let noctalia = noctalia(&defaults());
        assert_eq!(noctalia.strength().unwrap(), 80);
        assert_eq!(noctalia.kelvin(), Ok(Some(4000)));
        noctalia.set_strength(50).unwrap();
        assert_eq!(noctalia.config.get(NIGHT).unwrap(), "4750");
    }

    #[test]
    fn night_temperature_must_stay_below_day() {
        let noctalia = noctalia(&[(NIGHT, "4000"), (DAY, "5000")]);
        let error = noctalia.set_strength(0).unwrap_err();
        assert!(error.contains("100K below"), "{}", error);
        assert_eq!(noctalia.config.get(NIGHT).unwrap(), "4000");
    }

    struct FakeRunner {
        calls: RefCell<Vec<String>>,
        output: Result<String, String>,
    }

    impl Runner for FakeRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
            self.calls.borrow_mut().push(format!("{} {}", program, args.join(" ")));
            self.output.clone()
        }
    }

    fn config(output: Result<&str, &str>) -> NoctaliaConfig<FakeRunner> {
        NoctaliaConfig {
            runner: FakeRunner {
                calls: RefCell::new(vec![]),
                output: output.map(String::from).map_err(String::from),
            },
        }
    }

    #[test]
    fn config_goes_through_noctalia_ipc() {
        let config = config(Ok("'20:30'"));
        assert_eq!(config.read(SUNSET).unwrap(), Some("'20:30'".into()));
        config.write(SUNSET, "'21:00'").unwrap();
        assert_eq!(
            *config.runner.calls.borrow(),
            vec![
                "noctalia msg config-get location.sunset",
                "noctalia msg config-set location.sunset '21:00'",
            ]
        );
    }

    #[test]
    fn unset_paths_read_as_none_but_other_failures_are_errors() {
        let unset = config(Err("noctalia msg config-get location.sunset failed: error: path not found: location.sunset"));
        assert_eq!(unset.read(SUNSET), Ok(None));
        let down = config(Err("noctalia msg config-get nightlight.enabled failed: not running"));
        assert!(down.read(ENABLED).is_err());
    }
}
