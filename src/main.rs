use nightlight::{NightLight, Schedule, Status, Time};
use std::env::args;
use std::process::exit;

fn print_usage(program: &String) {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("  {}\n", env!("CARGO_PKG_DESCRIPTION"));
    println!("usage:\n  {} [--help] <command> [<args>]\n", program);
    println!("Available Commands By Category:");
    println!("\nmanual on/off control:");
    println!("  on                       Turn Night Shift on (until scheduled stop)");
    println!("  off                      Turn Night Shift off (until scheduled start)");
    println!("  status                   View current on/off status");
    println!("  toggle                   Toggle on or off based on current status");
    println!("  <noargs>                 Toggle on or off based on current status");
    println!("\ncolor temperature:");
    println!("  temp                     View temperature preference");
    println!("  temp <0-100|3500K-6000K> Set temperature preference (does not affect on/off)");
    println!("\nautomated schedule:");
    println!("  schedule                 View the current schedule");
    println!("  schedule start           Start schedule from sunset to sunrise");
    println!("  schedule <from> <to>     Start a custom schedule (12 or 24-hour time format)");
    println!("  schedule stop            Stop the current schedule");
    println!("  help                     Print this help menu");
}

fn print_status(client: NightLight) -> Result<(), String> {
    let schedule = client.get_schedule()?;
    let status = client.status()?;

    let off_at = match schedule {
        Schedule::SunsetToSunrise => " until sunrise".to_string(),
        Schedule::Off => "".to_string(),
        Schedule::Custom(from_time, to_time) => match status {
            Status::On => format!(" until {}", to_time),
            Status::Off => format!(" until {}", from_time),
        },
    };
    Ok(println!("{}{}", status, off_at))
}

fn toggle(client: NightLight) -> Result<(), String> {
    let status = client.status()?;

    match status {
        Status::On => client.off(),
        Status::Off => client.on(),
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    let command: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();

    let result = match command.as_slice() {
        ["help" | "-h" | "--help"] => {
            print_usage(&args[0]);
            Ok(())
        }
        [] | ["toggle"] => toggle(NightLight::new()),
        ["on"] => NightLight::new().on(),
        ["off"] => NightLight::new().off(),
        ["status"] => print_status(NightLight::new()),
        ["schedule"] => NightLight::new()
            .get_schedule()
            .map(|schedule| println!("{}", schedule)),
        ["schedule", "start"] => NightLight::new().set_schedule(Schedule::SunsetToSunrise),
        ["schedule", "stop"] => NightLight::new().set_schedule(Schedule::Off),
        ["schedule", from, to] => set_custom_schedule(
            NightLight::new(),
            &from.to_string(),
            &to.to_string(),
        ),
        ["temp"] => NightLight::new()
            .get_temp()
            .map(|temp| println!("{}", temp)),
        ["temp", value] => NightLight::new().set_temp(temp_userinput(value.to_string())),
        _ => Err(format!("Invalid arguments. Run '{} --help' for usage.", args[0])),
    };

    result.unwrap_or_else(|e| error(e));
}

fn temp_userinput(input: String) -> i32 {
    if let Some(temp) = input.parse().ok() {
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

fn set_custom_schedule(client: NightLight, from: &String, to: &String) -> Result<(), String> {
    let from = Time::parse(from)?;
    let to = Time::parse(to)?;

    client.set_schedule(Schedule::Custom(from, to))
}

fn error(text: String) {
    eprintln!("{}", text);
    exit(1)
}

#[cfg(test)]
mod tests {
    use super::temp_userinput;

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
