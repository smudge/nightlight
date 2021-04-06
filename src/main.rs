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
    println!("\ncolor temperature:");
    println!("  temp                     View temperature preference");
    println!("  temp <0-100|3500K-6500K> Set temperature preference (does not affect on/off)");
    println!("\nautomated schedule:");
    println!("  schedule                 View the current schedule");
    println!("  schedule start           Start schedule from sunset to sunrise");
    println!("  schedule <from> <to>     Start a custom schedule (12 or 24-hour time format)");
    println!("  schedule stop            Stop the current schedule");
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

    if args.len() < 2 {
        print_usage(&args[0]);
        exit(1);
    }

    let client = NightLight::new();
    if args.len() == 2 && args[1] == "on" {
        client.on().unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "off" {
        client.off().unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "toggle" {
        toggle(client).unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "schedule" {
        match client.get_schedule() {
            Ok(schedule) => println!("{}", schedule),
            Err(e) => error(e),
        }
    } else if args.len() == 3 && args[1] == "schedule" && args[2] == "start" {
        client
            .set_schedule(Schedule::SunsetToSunrise)
            .unwrap_or_else(|e| error(e));
    } else if args.len() == 4 && args[1] == "schedule" {
        set_custom_schedule(client, &args[2], &args[3]).unwrap_or_else(|e| error(e));
    } else if args.len() == 3 && args[1] == "schedule" && args[2] == "stop" {
        client
            .set_schedule(Schedule::Off)
            .unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "status" {
        print_status(client).unwrap_or_else(|e| error(e))
    } else if args.len() == 2 && args[1] == "temp" {
        match client.get_temp() {
            Ok(temp) => println!("{}", temp),
            Err(e) => error(e),
        }
    } else if args.len() == 3 && args[1] == "temp" {
        let temp = temp_userinput(args[2].clone());
        client.set_temp(temp).unwrap_or_else(|e| error(e));
    } else {
        print_usage(&args[0]);
    }
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
                    // Map kelvin value to 0-100
                    if kelvin_input < KELVIN_LOWER || kelvin_input > KELVIN_UPPER {
                        -1
                    }
                    else {
                        (((kelvin_input - KELVIN_LOWER) / (KELVIN_UPPER - KELVIN_LOWER)) * 100.0) as i32
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
