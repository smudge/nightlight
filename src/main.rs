extern crate objc;

use objc::rc::StrongPtr;
use objc::runtime::{Object, BOOL, YES};
use objc::{class, msg_send, sel, sel_impl};
use std::env::args;
use std::process::exit;

#[link(name = "CoreBrightness", kind = "framework")]
extern "C" {}

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
    println!("  temp [0-100]        Set color temperature to value");
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

struct NightShift {
    client: StrongPtr,
}

impl NightShift {
    pub fn new() -> NightShift {
        let client_class = class!(CBBlueLightClient);
        let client = unsafe {
            let obj: *mut Object = msg_send![client_class, alloc];
            let obj: *mut Object = msg_send![obj, init];
            StrongPtr::new(obj)
        };
        NightShift { client }
    }

    pub fn enable(&self, enabled: bool) {
        let enabled = enabled as BOOL;
        let result: BOOL = unsafe { msg_send![*self.client, setEnabled: enabled] };
        if result != (true as BOOL) {
            eprintln!("Failed to toggle Night Shift!");
            exit(1);
        }
    }

    pub fn set_temp(&self, temp: &String) {
        let temp = match temp.parse::<f32>() {
            Ok(v) => v / 100.0,
            Err(_) => {
                eprintln!("Invalid temperature value! Please choose a number between 0 and 100.");
                exit(1);
            }
        };
        let result: BOOL = unsafe { msg_send![*self.client, setStrength:temp commit:YES] };
        if result != (true as BOOL) {
            eprintln!("Failed to set color balance!");
            exit(1);
        }
    }
}
