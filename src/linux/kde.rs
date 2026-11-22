use super::command::{Runner, System};
use super::kelvin;
use super::transaction::{self, Store};
use crate::backend::Backend;
use crate::{Schedule, Status, Time};

const ACTIVE: &str = "kwinrc/NightColor/Active";
const MODE: &str = "kwinrc/NightColor/Mode";
const NIGHT: &str = "kwinrc/NightColor/NightTemperature";
const EVENING: &str = "kwinrc/NightColor/EveningBeginFixed";
const MORNING: &str = "kwinrc/NightColor/MorningBeginFixed";
const SOURCE: &str = "knighttimerc/General/Source";
const SUNSET: &str = "knighttimerc/Times/SunsetStart";
const SUNRISE: &str = "knighttimerc/Times/SunriseStart";

const DEFAULT_NIGHT: f64 = 4500.0;

const CONSTANT: &str = "KDE night light is set to always on, which has no schedule";

pub(crate) struct KConfig<R: Runner = System> {
    runner: R,
}

fn split(key: &str) -> (&str, &str, &str) {
    let mut parts = key.splitn(3, '/');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(file), Some(group), Some(key)) => (file, group, key),
        _ => unreachable!("malformed KDE config key {}", key),
    }
}

impl<R: Runner> Store for KConfig<R> {
    fn read(&self, key: &str) -> Result<Option<String>, String> {
        let (file, group, key) = split(key);
        let value = self.runner.run(
            "kreadconfig6",
            &["--file", file, "--group", group, "--key", key],
        )?;
        Ok(Some(value).filter(|value| !value.is_empty()))
    }

    fn write(&self, key: &str, value: &str) -> Result<(), String> {
        let (file, group, key) = split(key);
        self.runner
            .run(
                "kwriteconfig6",
                &["--file", file, "--group", group, "--key", key, "--notify", value],
            )
            .map(|_| ())
    }

    fn remove(&self, key: &str) -> Result<(), String> {
        let (file, group, key) = split(key);
        self.runner
            .run(
                "kwriteconfig6",
                &["--file", file, "--group", group, "--key", key, "--notify", "--delete"],
            )
            .map(|_| ())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Schema {
    /// Plasma 6.0-6.4
    Legacy,
    /// Plasma 6.5+
    NightTime,
}

fn parse_version(text: &str) -> Option<(u32, u32)> {
    text.split_whitespace().find_map(|word| {
        let mut parts = word.split('.');
        Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
    })
}

fn detect_schema(runner: &dyn Runner) -> Schema {
    ["kwin_wayland", "kwin_x11", "plasmashell"]
        .iter()
        .find_map(|program| parse_version(&runner.run(program, &["--version"]).ok()?))
        .map(|version| match version < (6, 5) {
            true => Schema::Legacy,
            false => Schema::NightTime,
        })
        .unwrap_or(Schema::NightTime)
}

pub(crate) struct Kde<S: Store = KConfig> {
    config: S,
    schema: Schema,
}

impl Kde {
    pub fn new() -> Kde {
        Kde {
            config: KConfig { runner: System },
            schema: detect_schema(&System),
        }
    }
}

fn parse_time(key: &str, value: &str) -> Result<Time, String> {
    let (hour, minute) = match value.split_once(':') {
        Some((hour, rest)) => (hour, rest.split(':').next().unwrap_or("")),
        None if value.len() == 4 && value.is_ascii() => value.split_at(2),
        None => ("", ""),
    };
    match (hour.parse(), minute.parse()) {
        (Ok(hour), Ok(minute)) => Time::from_tuple((hour, minute)),
        _ => Err(format!("Unexpected value for {}: {}", key, value)),
    }
}

fn legacy_time(time: Time) -> String {
    let (hour, minute) = time.tuple();
    format!("{:02}{:02}", hour, minute)
}

fn night_time(time: Time) -> String {
    let (hour, minute) = time.tuple();
    format!("{:02}:{:02}:00", hour, minute)
}

impl<S: Store> Kde<S> {
    fn value(&self, key: &str, default: &str) -> Result<String, String> {
        Ok(self.config.read(key)?.unwrap_or_else(|| default.to_string()))
    }

    fn active(&self) -> Result<bool, String> {
        Ok(self.value(ACTIVE, "false")? == "true")
    }

    fn time(&self, key: &str, default: &str) -> Result<Time, String> {
        parse_time(key, &self.value(key, default)?)
    }

    fn read_schedule(&self) -> Result<Option<Schedule>, String> {
        if !self.active()? {
            return Ok(Some(Schedule::Off));
        }
        Ok(match self.schema {
            Schema::Legacy => match self.value(MODE, "Automatic")?.as_str() {
                "Constant" => None,
                "Times" => Some(Schedule::Custom(
                    self.time(EVENING, "1800")?,
                    self.time(MORNING, "0600")?,
                )),
                _ => Some(Schedule::SunsetToSunrise),
            },
            Schema::NightTime => match self.value(MODE, "DarkLight")?.as_str() {
                "Constant" => None,
                _ => match self.value(SOURCE, "Location")?.as_str() {
                    "Times" => Some(Schedule::Custom(
                        self.time(SUNSET, "18:00:00")?,
                        self.time(SUNRISE, "06:00:00")?,
                    )),
                    _ => Some(Schedule::SunsetToSunrise),
                },
            },
        })
    }
}

impl<S: Store> Backend for Kde<S> {
    fn name(&self) -> &'static str {
        "kde"
    }

    fn status(&self) -> Result<Status, String> {
        Ok(Status::from_bool(self.active()?))
    }

    fn set_status(&self, status: Status) -> Result<(), String> {
        transaction::apply(&self.config, &[(ACTIVE, status.as_bool().to_string())])
    }

    fn has_schedule(&self) -> bool {
        !matches!(self.read_schedule(), Ok(None))
    }

    fn schedule(&self) -> Result<Schedule, String> {
        self.read_schedule()?.ok_or_else(|| CONSTANT.to_string())
    }

    fn set_schedule(&self, schedule: Schedule) -> Result<(), String> {
        let on = (ACTIVE, "true".to_string());
        let changes = match (schedule, self.schema) {
            (Schedule::Off, _) => vec![(ACTIVE, "false".into())],
            (Schedule::SunsetToSunrise, Schema::Legacy) => {
                // Don't switch manually entered coordinates to automatic.
                let mode = match self.value(MODE, "Automatic")?.as_str() {
                    "Location" => "Location",
                    _ => "Automatic",
                };
                vec![(MODE, mode.into()), on]
            }
            (Schedule::SunsetToSunrise, Schema::NightTime) => vec![
                (SOURCE, "Location".into()),
                (MODE, "DarkLight".into()),
                on,
            ],
            (Schedule::Custom(from, to), Schema::Legacy) => vec![
                (EVENING, legacy_time(from)),
                (MORNING, legacy_time(to)),
                (MODE, "Times".into()),
                on,
            ],
            (Schedule::Custom(from, to), Schema::NightTime) => vec![
                (SUNSET, night_time(from)),
                (SUNRISE, night_time(to)),
                (SOURCE, "Times".into()),
                (MODE, "DarkLight".into()),
                on,
            ],
        };
        transaction::apply(&self.config, &changes)
    }

    fn strength(&self) -> Result<i32, String> {
        let kelvin = match self.config.read(NIGHT)? {
            Some(value) => value
                .parse()
                .map_err(|_| format!("Unexpected value for {}: {}", NIGHT, value))?,
            None => DEFAULT_NIGHT,
        };
        Ok(kelvin::to_strength(kelvin))
    }

    fn set_strength(&self, strength: i32) -> Result<(), String> {
        let kelvin = kelvin::from_strength(strength);
        transaction::apply(&self.config, &[(NIGHT, kelvin.to_string())])
    }
}

#[cfg(test)]
mod tests {
    use super::super::transaction::fake::FakeStore;
    use super::*;
    use std::cell::RefCell;

    fn kde(schema: Schema, values: &[(&str, &str)]) -> Kde<FakeStore> {
        Kde {
            config: FakeStore::with(values),
            schema,
        }
    }

    fn time(hour: u8, minute: u8) -> Time {
        Time::from_tuple((hour, minute)).unwrap()
    }

    #[test]
    fn unset_config_uses_kwin_defaults() {
        for schema in [Schema::Legacy, Schema::NightTime] {
            let kde = kde(schema, &[]);
            assert_eq!(kde.status(), Ok(Status::Off));
            assert_eq!(kde.schedule(), Ok(Schedule::Off));
            assert_eq!(kde.strength(), Ok(60));
            kde.set_status(Status::On).unwrap();
            assert_eq!(kde.schedule(), Ok(Schedule::SunsetToSunrise));
        }
    }

    #[test]
    fn every_schedule_round_trips_on_both_schemas() {
        for schema in [Schema::Legacy, Schema::NightTime] {
            let kde = kde(schema, &[]);
            for schedule in [
                Schedule::Custom(time(19, 45), time(6, 30)),
                Schedule::SunsetToSunrise,
                Schedule::Off,
                Schedule::Custom(time(21, 0), time(7, 5)),
            ] {
                kde.set_schedule(schedule).unwrap();
                assert_eq!(kde.schedule(), Ok(schedule), "{:?}", schema);
            }
        }
    }

    #[test]
    fn legacy_custom_schedules_use_kwin_timings() {
        let kde = kde(Schema::Legacy, &[]);
        kde.set_schedule(Schedule::Custom(time(19, 45), time(6, 30))).unwrap();
        assert_eq!(kde.config.get(EVENING).unwrap(), "1945");
        assert_eq!(kde.config.get(MORNING).unwrap(), "0630");
        assert_eq!(kde.config.get(MODE).unwrap(), "Times");
        assert_eq!(kde.config.get(SUNSET), None);
    }

    #[test]
    fn knighttime_custom_schedules_use_knighttimerc() {
        let kde = kde(Schema::NightTime, &[(MODE, "Constant")]);
        kde.set_schedule(Schedule::Custom(time(19, 45), time(6, 30))).unwrap();
        assert_eq!(kde.config.get(SUNSET).unwrap(), "19:45:00");
        assert_eq!(kde.config.get(SUNRISE).unwrap(), "06:30:00");
        assert_eq!(kde.config.get(SOURCE).unwrap(), "Times");
        assert_eq!(kde.config.get(MODE).unwrap(), "DarkLight");
        assert_eq!(kde.config.get(EVENING), None);
    }

    #[test]
    fn legacy_sunset_keeps_manual_coordinates() {
        let kde = kde(Schema::Legacy, &[(ACTIVE, "true"), (MODE, "Location")]);
        assert_eq!(kde.schedule(), Ok(Schedule::SunsetToSunrise));
        kde.set_schedule(Schedule::SunsetToSunrise).unwrap();
        assert_eq!(kde.config.get(MODE).unwrap(), "Location");

        let kde = kde_times();
        kde.set_schedule(Schedule::SunsetToSunrise).unwrap();
        assert_eq!(kde.config.get(MODE).unwrap(), "Automatic");
    }

    fn kde_times() -> Kde<FakeStore> {
        kde(Schema::Legacy, &[(ACTIVE, "true"), (MODE, "Times")])
    }

    #[test]
    fn always_on_has_no_schedule_but_still_reports_status() {
        for schema in [Schema::Legacy, Schema::NightTime] {
            let kde = kde(schema, &[(ACTIVE, "true"), (MODE, "Constant")]);
            assert_eq!(kde.status(), Ok(Status::On));
            assert!(!kde.has_schedule());
            assert_eq!(kde.schedule(), Err(CONSTANT.to_string()));
        }
        assert!(kde(Schema::NightTime, &[(MODE, "Constant")]).has_schedule());
    }

    #[test]
    fn failed_activation_rolls_back_the_schedule() {
        let mut kde = kde(Schema::NightTime, &[]);
        kde.config.rejected_key = Some(ACTIVE);
        assert!(kde.set_schedule(Schedule::Custom(time(19, 45), time(6, 30))).is_err());
        assert!(kde.config.values.borrow().is_empty(), "{:?}", kde.config.values);
    }

    #[test]
    fn strength_maps_onto_the_night_temperature() {
        let kde = kde(Schema::NightTime, &[(NIGHT, "3000")]);
        assert_eq!(kde.strength(), Ok(100));
        kde.set_strength(50).unwrap();
        assert_eq!(kde.config.get(NIGHT).unwrap(), "4750");
    }

    #[test]
    fn times_parse_in_both_formats() {
        assert_eq!(parse_time("k", "0630"), Ok(time(6, 30)));
        assert_eq!(parse_time("k", "19:45:00"), Ok(time(19, 45)));
        assert_eq!(parse_time("k", "7:05"), Ok(time(7, 5)));
        assert!(parse_time("k", "630").is_err());
        assert!(parse_time("k", "25:00:00").is_err());
    }

    struct FakeRunner {
        calls: RefCell<Vec<String>>,
        outputs: Vec<(&'static str, Result<&'static str, &'static str>)>,
    }

    impl Runner for FakeRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
            let call = format!("{} {}", program, args.join(" "));
            self.calls.borrow_mut().push(call.clone());
            self.outputs
                .iter()
                .find(|(prefix, _)| call.starts_with(prefix))
                .map(|(_, output)| output.map(String::from).map_err(String::from))
                .unwrap_or_else(|| Err(format!("{}: not found", program)))
        }
    }

    fn runner(outputs: Vec<(&'static str, Result<&'static str, &'static str>)>) -> FakeRunner {
        FakeRunner {
            calls: RefCell::new(vec![]),
            outputs,
        }
    }

    #[test]
    fn kconfig_tools_are_invoked_with_notify() {
        let config = KConfig {
            runner: runner(vec![("kreadconfig6", Ok("")), ("kwriteconfig6", Ok(""))]),
        };
        assert_eq!(config.read(SUNSET), Ok(None));
        config.write(ACTIVE, "true").unwrap();
        config.remove(SUNSET).unwrap();
        assert_eq!(
            *config.runner.calls.borrow(),
            vec![
                "kreadconfig6 --file knighttimerc --group Times --key SunsetStart",
                "kwriteconfig6 --file kwinrc --group NightColor --key Active --notify true",
                "kwriteconfig6 --file knighttimerc --group Times --key SunsetStart --notify --delete",
            ]
        );
    }

    #[test]
    fn schema_follows_the_kwin_version() {
        let detect = |outputs| detect_schema(&runner(outputs));
        assert_eq!(detect(vec![("kwin_wayland", Ok("kwin 6.4.5"))]), Schema::Legacy);
        assert_eq!(detect(vec![("kwin_wayland", Ok("kwin 6.5.0"))]), Schema::NightTime);
        assert_eq!(detect(vec![("kwin_wayland", Ok("kwin_wayland 6.8.3"))]), Schema::NightTime);
        assert_eq!(detect(vec![("kwin_x11", Ok("kwin 6.3.6"))]), Schema::Legacy);
        assert_eq!(detect(vec![("plasmashell", Ok("plasmashell 6.2.0"))]), Schema::Legacy);
        assert_eq!(detect(vec![]), Schema::NightTime);
    }
}
