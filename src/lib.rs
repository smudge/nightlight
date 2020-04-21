extern crate objc;

use objc::rc::StrongPtr;
use objc::runtime::{Object, BOOL, YES};
use objc::{class, msg_send, sel, sel_impl};
use std::process::exit; // TODO: bubble errors up to main.rs

#[link(name = "CoreBrightness", kind = "framework")]
extern "C" {}

pub struct NightShift {
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
