use nightshift::NightShift;
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
    println!("  on                  Turn Night Shift on (until tomorrow/sunrise)");
    println!("  off                 Turn Night Shift off");
    println!("  temp [0-100]        Set color temperature preference (does not affect on/off)");
}

fn print_status(status: nightshift::ffi::Status) {
    println!("active: {}", status.active());
    println!("enabled: {}", status.enabled());
    println!(
        "sun_schedule_permitted: {}",
        status.sun_schedule_permitted()
    );
    println!("mode: {}", status.mode());
    println!("schedule: {} - {}", status.from_time(), status.to_time());
    println!("disable_flags: {}", status.disable_flags());
    println!("available: {}", status.available());
}

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        exit(1);
    }

    let night_shift = NightShift::new();
    if args.len() == 2 && args[1] == "on" {
        night_shift.on().unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "off" {
        night_shift.off().unwrap_or_else(|e| error(e));
    } else if args.len() == 2 && args[1] == "status" {
        match night_shift.status() {
            Ok(s) => print_status(s),
            Err(e) => error(e),
        }
    } else if args.len() == 3 && args[1] == "temp" {
        let temp = args[2].parse().unwrap_or(-1);
        night_shift.set_temp(temp).unwrap_or_else(|e| error(e));
    } else {
        print_usage(&args[0]);
    }
}

fn error(text: String) {
    eprintln!("{}", text);
    exit(1)
}
