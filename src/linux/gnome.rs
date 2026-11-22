use super::command::{Runner, System};
use super::transaction::{self, Store};
use crate::backend::Backend;
use crate::{Schedule, Status, Time};

const SCHEMA: &str = "org.gnome.settings-daemon.plugins.color";
const ENABLED: &str = "night-light-enabled";
const AUTOMATIC: &str = "night-light-schedule-automatic";
const FROM: &str = "night-light-schedule-from";
const TO: &str = "night-light-schedule-to";
const TEMPERATURE: &str = "night-light-temperature";

const KELVIN_WARMEST: f64 = 3500.0;
const KELVIN_COOLEST: f64 = 6000.0;

pub(crate) struct GSettings<R: Runner = System> {
    runner: R,
}

impl GSettings {
    pub fn new() -> GSettings {
        GSettings { runner: System }
    }
}

impl<R: Runner> Store for GSettings<R> {
    fn read(&self, key: &str) -> Result<String, String> {
        self.runner.run("gsettings", &["get", SCHEMA, key])
    }

    fn write(&self, key: &str, value: &str) -> Result<(), String> {
        self.runner
            .run("gsettings", &["set", SCHEMA, key, value])
            .map(|_| ())
            .map_err(|_| format!("Unable to set {}", key))
    }
}

pub(crate) fn installed(runner: &dyn Runner) -> bool {
    runner.run("gsettings", &["get", SCHEMA, ENABLED]).is_ok()
}

pub(crate) struct Gnome<S: Store = GSettings> {
    settings: S,
}

impl Gnome {
    pub fn new() -> Gnome {
        Gnome {
            settings: GSettings::new(),
        }
    }
}

// `gsettings get` prefixes some values with their type, e.g. "uint32 2700".
fn value(text: &str) -> &str {
    text.split_whitespace().last().unwrap_or("")
}

fn parse<T: std::str::FromStr>(key: &str, text: &str) -> Result<T, String> {
    value(text)
        .parse()
        .map_err(|_| format!("Unexpected value for {}: {}", key, text))
}

fn hours_to_time(hours: f64) -> Result<Time, String> {
    Time::from_tuple((hours.trunc() as u8, (hours.fract() * 60.0).round() as u8))
}

fn time_to_hours(time: Time) -> String {
    let (hour, minute) = time.tuple();
    format!("{:?}", hour as f64 + minute as f64 / 60.0)
}

impl<S: Store> Gnome<S> {
    fn read<T: std::str::FromStr>(&self, key: &str) -> Result<T, String> {
        parse(key, &self.settings.read(key)?)
    }
}

impl<S: Store> Backend for Gnome<S> {
    fn name(&self) -> &'static str {
        "gnome"
    }

    fn status(&self) -> Result<Status, String> {
        Ok(Status::from_bool(self.read(ENABLED)?))
    }

    fn set_status(&self, status: Status) -> Result<(), String> {
        self.settings.write(ENABLED, &status.as_bool().to_string())
    }

    fn schedule(&self) -> Result<Schedule, String> {
        if !self.read::<bool>(ENABLED)? {
            return Ok(Schedule::Off);
        }
        if self.read::<bool>(AUTOMATIC)? {
            return Ok(Schedule::SunsetToSunrise);
        }
        Ok(Schedule::Custom(
            hours_to_time(self.read(FROM)?)?,
            hours_to_time(self.read(TO)?)?,
        ))
    }

    fn set_schedule(&self, schedule: Schedule) -> Result<(), String> {
        let changes = match schedule {
            Schedule::Off => vec![(ENABLED, "false".into()), (AUTOMATIC, "false".into())],
            Schedule::SunsetToSunrise => {
                vec![(ENABLED, "true".into()), (AUTOMATIC, "true".into())]
            }
            Schedule::Custom(from, to) => vec![
                (FROM, time_to_hours(from)),
                (TO, time_to_hours(to)),
                (ENABLED, "true".into()),
                (AUTOMATIC, "false".into()),
            ],
        };
        transaction::apply(&self.settings, &changes)
    }

    fn strength(&self) -> Result<i32, String> {
        let kelvins: f64 = self.read(TEMPERATURE)?;
        let ratio = 1.0 - (kelvins - KELVIN_WARMEST) / (KELVIN_COOLEST - KELVIN_WARMEST);
        Ok((ratio * 100.0) as i32)
    }

    fn set_strength(&self, strength: i32) -> Result<(), String> {
        let ratio = strength as f64 / 100.0;
        let kelvins = (1.0 - ratio) * (KELVIN_COOLEST - KELVIN_WARMEST) + KELVIN_WARMEST;
        self.settings
            .write(TEMPERATURE, &(kelvins as u32).to_string())
            .map_err(|_| "Unable to set temperature".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::super::transaction::fake::FakeStore;
    use super::*;
    use std::cell::RefCell;

    fn gnome(values: &[(&str, &str)]) -> Gnome<FakeStore> {
        Gnome {
            settings: FakeStore::with(values),
        }
    }

    fn defaults() -> Vec<(&'static str, &'static str)> {
        vec![
            (ENABLED, "false"),
            (AUTOMATIC, "true"),
            (FROM, "20.0"),
            (TO, "6.0"),
            (TEMPERATURE, "uint32 4750"),
        ]
    }

    fn time(hour: u8, minute: u8) -> Time {
        Time::from_tuple((hour, minute)).unwrap()
    }

    #[test]
    fn every_schedule_round_trips() {
        let gnome = gnome(&defaults());
        for schedule in [
            Schedule::SunsetToSunrise,
            Schedule::Off,
            Schedule::Custom(time(7, 30), time(21, 15)),
            Schedule::SunsetToSunrise,
        ] {
            gnome.set_schedule(schedule).unwrap();
            assert_eq!(gnome.schedule().unwrap(), schedule);
        }
    }

    #[test]
    fn disabled_night_light_is_not_reported_as_custom() {
        let gnome = gnome(&[(ENABLED, "false"), (AUTOMATIC, "false")]);
        assert_eq!(gnome.schedule().unwrap(), Schedule::Off);
    }

    #[test]
    fn stopping_schedule_disables_night_light() {
        let gnome = gnome(&defaults());
        gnome.set_schedule(Schedule::SunsetToSunrise).unwrap();
        gnome.set_schedule(Schedule::Off).unwrap();
        assert_eq!(gnome.status().unwrap(), Status::Off);
        assert_eq!(gnome.settings.get(AUTOMATIC).unwrap(), "false");
    }

    #[test]
    fn custom_schedules_store_fractional_hours() {
        let gnome = gnome(&defaults());
        gnome
            .set_schedule(Schedule::Custom(time(7, 30), time(21, 0)))
            .unwrap();
        assert_eq!(gnome.settings.get(FROM).unwrap(), "7.5");
        assert_eq!(gnome.settings.get(TO).unwrap(), "21.0");
    }

    #[test]
    fn non_custom_schedules_do_not_read_times() {
        let gnome = gnome(&[(ENABLED, "true"), (AUTOMATIC, "true"), (FROM, "99.0")]);
        assert_eq!(gnome.schedule().unwrap(), Schedule::SunsetToSunrise);
    }

    #[test]
    fn invalid_custom_times_are_errors() {
        let gnome = gnome(&[(ENABLED, "true"), (AUTOMATIC, "false"), (FROM, "24.0"), (TO, "6.0")]);
        assert!(gnome.schedule().is_err());
    }

    #[test]
    fn failed_mode_write_discards_custom_times() {
        let mut gnome = gnome(&defaults());
        gnome.settings.rejected_key = Some(AUTOMATIC);
        assert_eq!(
            gnome.set_schedule(Schedule::Custom(time(7, 30), time(21, 15))),
            Err(format!("Unable to set {}", AUTOMATIC))
        );
        gnome.settings.rejected_key = None;
        assert_eq!(gnome.schedule().unwrap(), Schedule::Off);
        assert_eq!(gnome.settings.get(FROM).unwrap(), "20.0");
        assert_eq!(gnome.settings.get(TO).unwrap(), "6.0");
    }

    #[test]
    fn mode_only_update_preserves_times() {
        let gnome = gnome(&defaults());
        gnome
            .set_schedule(Schedule::Custom(time(7, 30), time(21, 15)))
            .unwrap();
        gnome.set_schedule(Schedule::Off).unwrap();
        assert_eq!(gnome.settings.get(FROM).unwrap(), "7.5");
    }

    #[test]
    fn status_toggles_only_the_enabled_key() {
        let gnome = gnome(&defaults());
        gnome.set_status(Status::On).unwrap();
        assert_eq!(gnome.status().unwrap(), Status::On);
        assert_eq!(gnome.settings.writes.borrow().len(), 1);
    }

    #[test]
    fn strength_maps_onto_kelvin() {
        let gnome = gnome(&defaults());
        assert_eq!(gnome.strength().unwrap(), 50);
        gnome.set_strength(100).unwrap();
        assert_eq!(gnome.settings.get(TEMPERATURE).unwrap(), "3500");
        gnome.set_strength(0).unwrap();
        assert_eq!(gnome.settings.get(TEMPERATURE).unwrap(), "6000");
    }

    struct FakeRunner {
        calls: RefCell<Vec<Vec<String>>>,
        output: Result<String, String>,
    }

    impl Runner for FakeRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
            let mut call = vec![program.to_string()];
            call.extend(args.iter().map(|arg| arg.to_string()));
            self.calls.borrow_mut().push(call);
            self.output.clone()
        }
    }

    #[test]
    fn gsettings_invocations_target_the_color_schema() {
        let settings = GSettings {
            runner: FakeRunner {
                calls: RefCell::new(vec![]),
                output: Ok("uint32 2700".into()),
            },
        };
        assert_eq!(settings.read(TEMPERATURE).unwrap(), "uint32 2700");
        settings.write(ENABLED, "true").unwrap();
        assert_eq!(
            *settings.runner.calls.borrow(),
            vec![
                vec!["gsettings", "get", SCHEMA, TEMPERATURE],
                vec!["gsettings", "set", SCHEMA, ENABLED, "true"],
            ]
        );
    }

    #[test]
    fn gsettings_write_failures_name_the_key() {
        let settings = GSettings {
            runner: FakeRunner {
                calls: RefCell::new(vec![]),
                output: Err("No such schema".into()),
            },
        };
        assert_eq!(settings.write(TO, "6.0"), Err(format!("Unable to set {}", TO)));
    }
}
