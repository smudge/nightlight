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
    println!("  on                  Turns Night Shift on (until tomorrow/sunrise)");
    println!("  off                 Turns Night Shift off");
    println!("  temp [0-100]        Set color temperature preference (does not affect on/off)");
}

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        exit(1);
    }

    let night_shift = NightShift::new();
    if args.len() == 2 && args[1] == "on" {
        night_shift.enable(true);
    } else if args.len() == 2 && args[1] == "off" {
        night_shift.enable(false);
    } else if args.len() == 3 && args[1] == "temp" {
        night_shift.set_temp(&args[2]);
    } else {
        print_usage(&args[0]);
    }
}
