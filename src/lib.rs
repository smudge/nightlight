extern crate objc;

use objc::rc::StrongPtr;
use objc::runtime::{Object, BOOL, YES};
use objc::{class, msg_send, sel, sel_impl};

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

    pub fn enable(&self, enabled: bool) -> Result<(), String> {
        let result: BOOL = unsafe { msg_send![*self.client, setEnabled: (enabled as BOOL)] };
        if result == (true as BOOL) {
            Ok(())
        } else {
            Err(format!("Failed to turn Night Shift {}", on_or_off(enabled)))
        }
    }

    pub fn set_temp(&self, temp: &String) -> Result<(), String> {
        let temp = match temp.parse::<f32>() {
            Ok(v) => v / 100.0,
            Err(_) => {
                return Err("Color temperature must be a number from 0 to 100.".to_string());
            }
        };
        let result: BOOL = unsafe { msg_send![*self.client, setStrength:temp commit:YES] };

        if result == (true as BOOL) {
            Ok(())
        } else {
            Err("Failed to set color temperature".to_string())
        }
    }
}

fn on_or_off(value: bool) -> String {
    if value {
        "on".to_string()
    } else {
        "off".to_string()
    }
}
