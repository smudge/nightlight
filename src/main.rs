use nightlight::{NightLight, Schedule, Status, Time};
use std::env::args;
use std::process::exit;

fn print_usage(program: &String) {
    println!(
        "{} (version {})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("{}\n", env!("CARGO_PKG_DESCRIPTION"));
    println!("Usage:\n  {} [command]\n", program);
    println!("Available Commands:");
    println!("  on                      Turn Night Shift on (until sunrise/scheduled stop)");
    println!("  off                     Turn Night Shift off (until sunset/scheduled start)");
    println!("  schedule                Start schedule from sunset to sunrise");
    println!("  schedule [from] [to]    Start a custom schedule (12 or 24-hour time format)");
    println!("  unschedule              Stop schedule");
    println!("  temp [0-100]            Set color temperature preference (does not affect on/off)");
    println!("  status                  View current status and configuration");
}

fn print_status(status: Status) {
    println!("Schedule:\n=> {}", status.schedule);
    let off_at = match status.schedule {
        Schedule::SunsetToSunrise => "Sunrise",
        Schedule::Off => "Tomorrow",
        Schedule::Custom(from_time, to_time) => {
            println!(
                "From:\n=> {} to {}",
                from_time.to_string(),
                to_time.to_string()
            );
            "Tomorrow"
        }
    };
    println!("On Until {}:\n=> {}", off_at, status.currently_active);
    println!("Color Temperature:\n=> {}", status.color_temperature);
}

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        exit(1);
    }

    let night_shift = NightLight::new();
    if args.len() == 2 && args[1] == "on" {
        night_shift.on().unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "off" {
        night_shift.off().unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "schedule" {
        night_shift
            .set_schedule(Schedule::SunsetToSunrise)
            .unwrap_or_else(|e| error(e));
    } else if args.len() == 4 && args[1] == "schedule" {
        schedule(night_shift, &args[2], &args[3]).unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "unschedule" {
        night_shift
            .set_schedule(Schedule::Off)
            .unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "status" {
        match night_shift.status() {
            Ok(status) => print_status(status),
            Err(e) => error(e),
        }
    } else if args.len() == 3 && args[1] == "temp" {
        let temp = args[2].parse().unwrap_or(-1);
        night_shift.set_temp(temp).unwrap_or_else(|e| error(e));
    } else {
        print_usage(&args[0]);
    }
}

fn schedule(night_shift: NightLight, from: &String, to: &String) -> Result<(), String> {
    let from = Time::parse(from)?;
    let to = Time::parse(to)?;

    night_shift.set_schedule(Schedule::Custom(from, to))
}

fn error(text: String) {
    eprintln!("{}", text);
    exit(1)
}
