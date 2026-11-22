use nightlight::{NightLight, Schedule, Status, Time};
use serde_json::{json, Value};
use std::env::args;
use std::process::exit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

enum Command<'a> {
    Help,
    Toggle,
    On,
    Off,
    Status,
    Schedule,
    ScheduleStart,
    ScheduleStop,
    ScheduleCustom(&'a str, &'a str),
    Temperature,
    SetTemperature(&'a str),
}

enum CommandOutput {
    Help,
    Status(Status, Option<Schedule>),
    Schedule(Schedule),
    Temperature(i32),
    Applied {
        command: &'static str,
        value: Value,
    },
}

#[cfg(target_os = "linux")]
const ENVIRONMENT: &str = r#"
Environment:
  NIGHTLIGHT_BACKEND       Use a specific backend instead of detecting one
"#;
#[cfg(not(target_os = "linux"))]
const ENVIRONMENT: &str = "";

fn usage_text(program: &str) -> String {
    format!(
        r#"{} v{}
  {}

usage:
  {} [--json] <command> [<args>]

Global options:
  --json                   Output JSON for any command (including errors)
{}
Available Commands By Category:

manual on/off control:
  on                       Turn Night Shift on (until scheduled stop)
  off                      Turn Night Shift off (until scheduled start)
  status                   View current on/off status
  toggle                   Toggle on or off based on current status
  <noargs>                 Toggle on or off based on current status

color temperature:
  temp                     View temperature preference
  temp <0-100|3500K-6000K> Set temperature preference (does not affect on/off)

automated schedule:
  schedule                 View the current schedule
  schedule start           Start schedule from sunset to sunrise
  schedule <from> <to>     Start a custom schedule (12 or 24-hour time format)
  schedule stop            Stop the current schedule
  help                     Print this help menu"#,
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION"),
        program,
        ENVIRONMENT
    )
}

fn parse_global_options<'a>(args: &[&'a str]) -> (OutputFormat, Vec<&'a str>) {
    let format = match args.contains(&"--json") {
        true => OutputFormat::Json,
        false => OutputFormat::Text,
    };
    let command = args.iter().copied().filter(|arg| *arg != "--json").collect();
    (format, command)
}

fn parse_command<'a>(args: &[&'a str], program: &str) -> Result<Command<'a>, String> {
    match args {
        ["help" | "-h" | "--help"] => Ok(Command::Help),
        [] | ["toggle"] => Ok(Command::Toggle),
        ["on"] => Ok(Command::On),
        ["off"] => Ok(Command::Off),
        ["status"] => Ok(Command::Status),
        ["schedule"] => Ok(Command::Schedule),
        ["schedule", "start"] => Ok(Command::ScheduleStart),
        ["schedule", "stop"] => Ok(Command::ScheduleStop),
        ["schedule", from, to] => Ok(Command::ScheduleCustom(from, to)),
        ["temp"] => Ok(Command::Temperature),
        ["temp", value] => Ok(Command::SetTemperature(value)),
        _ => Err(format!("Invalid arguments. Run '{} --help' for usage.", program)),
    }
}

fn schedule_value(schedule: &Schedule) -> Value {
    match schedule {
        Schedule::Off => json!({"type": "off"}),
        Schedule::SunsetToSunrise => json!({"type": "sunset-to-sunrise"}),
        Schedule::Custom(from, to) => {
            let (from_hour, from_minute) = from.tuple();
            let (to_hour, to_minute) = to.tuple();
            json!({
                "type": "custom",
                "from": format!("{:02}:{:02}", from_hour, from_minute),
                "to": format!("{:02}:{:02}", to_hour, to_minute)
            })
        }
    }
}

fn status_text(status: &Status, schedule: &Option<Schedule>) -> String {
    let suffix = match (schedule, status) {
        (Some(Schedule::SunsetToSunrise), Status::On) => " until sunrise".to_string(),
        (Some(Schedule::SunsetToSunrise), Status::Off) => " until sunset".to_string(),
        (Some(Schedule::Off), _) | (None, _) => String::new(),
        (Some(Schedule::Custom(_, to)), Status::On) => format!(" until {}", to),
        (Some(Schedule::Custom(from, _)), Status::Off) => format!(" until {}", from),
    };
    format!("{}{}", status, suffix)
}

fn output_value(output: &CommandOutput, program: &str) -> Value {
    match output {
        CommandOutput::Help => json!({"ok": true, "help": usage_text(program)}),
        CommandOutput::Status(status, schedule) => json!({
            "status": status.to_string(),
            "schedule": schedule.as_ref().map(schedule_value)
        }),
        CommandOutput::Schedule(schedule) => json!({"schedule": schedule_value(schedule)}),
        CommandOutput::Temperature(temp) => json!({"temperature": temp}),
        CommandOutput::Applied { command, value } => json!({
            "ok": true,
            "command": command,
            "value": value
        }),
    }
}

fn format_output(output: &CommandOutput, format: OutputFormat, program: &str) -> Option<String> {
    match (format, output) {
        (OutputFormat::Json, output) => Some(output_value(output, program).to_string()),
        (OutputFormat::Text, CommandOutput::Help) => Some(usage_text(program)),
        (OutputFormat::Text, CommandOutput::Status(status, schedule)) => {
            Some(status_text(status, schedule))
        }
        (OutputFormat::Text, CommandOutput::Schedule(schedule)) => Some(schedule.to_string()),
        (OutputFormat::Text, CommandOutput::Temperature(temp)) => Some(temp.to_string()),
        (OutputFormat::Text, CommandOutput::Applied { .. }) => None,
    }
}

fn format_error(message: &str, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => message.to_owned(),
        OutputFormat::Json => json!({"ok": false, "error": message}).to_string(),
    }
}

fn execute(command: Command<'_>) -> Result<CommandOutput, String> {
    match command {
        Command::Help => Ok(CommandOutput::Help),
        Command::Toggle => {
            let client = NightLight::new();
            match client.status()? {
                Status::On => {
                    client.off()?;
                    Ok(CommandOutput::Applied {
                        command: "toggle",
                        value: json!({"status": "off"}),
                    })
                }
                Status::Off => {
                    client.on()?;
                    Ok(CommandOutput::Applied {
                        command: "toggle",
                        value: json!({"status": "on"}),
                    })
                }
            }
        }
        Command::On => {
            NightLight::new().on()?;
            Ok(CommandOutput::Applied {
                command: "on",
                value: json!({"status": "on"}),
            })
        }
        Command::Off => {
            NightLight::new().off()?;
            Ok(CommandOutput::Applied {
                command: "off",
                value: json!({"status": "off"}),
            })
        }
        Command::Status => {
            let client = NightLight::new();
            let schedule = client.current_schedule()?;
            let status = client.status()?;
            Ok(CommandOutput::Status(status, schedule))
        }
        Command::Schedule => {
            Ok(CommandOutput::Schedule(NightLight::new().get_schedule()?))
        }
        Command::ScheduleStart => {
            let schedule = Schedule::SunsetToSunrise;
            let value = schedule_value(&schedule);
            NightLight::new().set_schedule(schedule)?;
            Ok(CommandOutput::Applied { command: "schedule", value })
        }
        Command::ScheduleStop => {
            let schedule = Schedule::Off;
            let value = schedule_value(&schedule);
            NightLight::new().set_schedule(schedule)?;
            Ok(CommandOutput::Applied { command: "schedule", value })
        }
        Command::ScheduleCustom(from, to) => {
            let from = Time::parse(&from.to_string())?;
            let to = Time::parse(&to.to_string())?;
            let schedule = Schedule::Custom(from, to);
            let value = schedule_value(&schedule);
            NightLight::new().set_schedule(schedule)?;
            Ok(CommandOutput::Applied { command: "schedule", value })
        }
        Command::Temperature => {
            Ok(CommandOutput::Temperature(NightLight::new().get_temp()?))
        }
        Command::SetTemperature(input) => match temp_userinput(input.to_string()) {
            temp @ 0..=100 => {
                NightLight::new().set_temp(temp)?;
                Ok(CommandOutput::Applied {
                    command: "temp",
                    value: json!({"temperature": temp}),
                })
            }
            _ => Err("Color temperature must be a number from 0 to 100.".into()),
        },
    }
}

fn main() {
    let argv: Vec<String> = args().collect();
    let program = argv.first().map(String::as_str).unwrap_or("nightlight");
    let args: Vec<&str> = argv.iter().skip(1).map(String::as_str).collect();
    let (format, command_args) = parse_global_options(&args);
    let result = parse_command(&command_args, program).and_then(execute);

    match result {
        Ok(output) => {
            if let Some(text) = format_output(&output, format, program) {
                println!("{}", text);
            }
        }
        Err(message) => {
            eprintln!("{}", format_error(&message, format));
            exit(1);
        }
    }
}

fn temp_userinput(input: String) -> i32 {
    if let Ok(temp) = input.parse() {
        temp
    } else {
        const KELVIN_LOWER: f64 = 3500.0;
        const KELVIN_UPPER: f64 = 6000.0;

        if input.to_ascii_uppercase().ends_with("K") {
            match input[..(input.len() - 1)].parse::<f64>().ok() {
                Some(kelvin_input) => {
                    if !kelvin_input.is_finite()
                        || !(KELVIN_LOWER..=KELVIN_UPPER).contains(&kelvin_input)
                    {
                        -1
                    } else {
                        (((KELVIN_UPPER - kelvin_input) / (KELVIN_UPPER - KELVIN_LOWER)) * 100.0) as i32
                    }
                }
                None => -1,
            }
        } else {
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn global_format_is_independent_of_command_position() {
        for args in [
            &["--json", "schedule", "7:30", "21:00"][..],
            &["schedule", "--json", "7:30", "21:00"][..],
            &["schedule", "7:30", "21:00", "--json"][..],
        ] {
            let (format, arguments) = parse_global_options(args);
            assert_eq!(format, OutputFormat::Json);
            assert!(matches!(parse_command(&arguments, "nightlight"),
                Ok(Command::ScheduleCustom("7:30", "21:00"))));
        }
        let (format, arguments) = parse_global_options(&["status"]);
        assert_eq!(format, OutputFormat::Text);
        assert!(matches!(parse_command(&arguments, "nightlight"), Ok(Command::Status)));
    }

    #[test]
    fn global_format_applies_to_all_command_shapes() {
        for args in [
            &["on"][..], &["off"][..], &["toggle"][..], &["status"][..],
            &["schedule"][..], &["schedule", "start"][..],
            &["schedule", "stop"][..], &["temp"][..],
            &["temp", "70"][..], &["help"][..],
        ] {
            let mut input = args.to_vec();
            input.push("--json");
            let (format, command) = parse_global_options(&input);
            assert_eq!(format, OutputFormat::Json);
            assert!(parse_command(&command, "nightlight").is_ok(), "{:?}", args);
        }
    }

    #[test]
    fn json_output_for_status_and_schedules() {
        assert_eq!(
            output_value(&CommandOutput::Status(Status::Off, Some(Schedule::Off)), "nightlight"),
            json!({"status":"off","schedule":{"type":"off"}})
        );
        assert_eq!(
            output_value(&CommandOutput::Status(Status::On, Some(Schedule::SunsetToSunrise)), "nightlight"),
            json!({"status":"on","schedule":{"type":"sunset-to-sunrise"}})
        );
        assert_eq!(
            output_value(&CommandOutput::Status(Status::On, None), "nightlight"),
            json!({"status":"on","schedule":null})
        );
        assert_eq!(status_text(&Status::On, &None), "on");
        let schedule = Schedule::Custom(
            Time::from_tuple((7, 5)).unwrap(),
            Time::from_tuple((21, 30)).unwrap(),
        );
        assert_eq!(
            output_value(&CommandOutput::Status(Status::Off, Some(schedule)), "nightlight"),
            json!({
                "status":"off",
                "schedule":{"type":"custom","from":"07:05","to":"21:30"}
            })
        );
        assert_eq!(
            output_value(&CommandOutput::Temperature(75), "nightlight"),
            json!({"temperature":75})
        );
    }

    #[test]
    fn status_text_names_the_next_transition() {
        let sunset = Some(Schedule::SunsetToSunrise);
        assert_eq!(status_text(&Status::On, &sunset), "on until sunrise");
        assert_eq!(status_text(&Status::Off, &sunset), "off until sunset");
        assert_eq!(status_text(&Status::Off, &Some(Schedule::Off)), "off");
        let (from, to) = (Time::from_tuple((20, 0)).unwrap(), Time::from_tuple((6, 30)).unwrap());
        let (on, off) = (format!("on until {}", to), format!("off until {}", from));
        let custom = Some(Schedule::Custom(from, to));
        assert_eq!(status_text(&Status::On, &custom), on);
        assert_eq!(status_text(&Status::Off, &custom), off);
    }

    #[test]
    fn json_output_for_mutations_and_help() {
        let changed = CommandOutput::Applied {
            command: "temp",
            value: json!({"temperature": 70}),
        };
        assert_eq!(
            output_value(&changed, "nightlight"),
            json!({"ok":true,"command":"temp","value":{"temperature":70}})
        );
        assert!(format_output(&changed, OutputFormat::Text, "nightlight").is_none());
        assert!(output_value(&CommandOutput::Help, "nightlight")["help"]
            .as_str().unwrap().contains("--json"));
    }

    #[test]
    fn json_errors_escape_messages() {
        let message = "bad \"input\"\nnext line";
        let json: Value = serde_json::from_str(&format_error(message, OutputFormat::Json)).unwrap();
        assert_eq!(json, json!({"ok":false,"error":message}));
        assert_eq!(format_error(message, OutputFormat::Text), message);
    }

    #[test]
    fn kelvin_endpoints_match_strength() {
        assert_eq!(temp_userinput("3500K".into()), 100);
        assert_eq!(temp_userinput("6000K".into()), 0);
        assert_eq!(temp_userinput("4750k".into()), 50);
    }

    #[test]
    fn kelvin_input_rejects_out_of_range_values() {
        for value in ["3499K", "6001K", "6500K", "NaNK", "infK", "invalidK"] {
            assert_eq!(temp_userinput(value.into()), -1, "{}", value);
        }
    }

    #[test]
    fn percentage_input_is_unchanged() {
        assert_eq!(temp_userinput("0".into()), 0);
        assert_eq!(temp_userinput("70".into()), 70);
        assert_eq!(temp_userinput("100".into()), 100);
    }
}
