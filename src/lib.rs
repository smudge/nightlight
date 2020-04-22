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

    pub fn on(&self) -> Result<(), String> {
        self.toggle(true)
    }

    pub fn off(&self) -> Result<(), String> {
        self.toggle(false)
    }

    pub fn toggle(&self, on: bool) -> Result<(), String> {
        let result: BOOL = unsafe { msg_send![*self.client, setEnabled: (on as BOOL)] };
        if result == (true as BOOL) {
            Ok(())
        } else {
            Err(format!("Failed to turn Night Shift {}", on_or_off(on)))
        }
    }

    pub fn set_temp(&self, temp: i32) -> Result<(), String> {
        if temp < 0 || temp > 100 {
            return Err("Color temperature must be a number from 0 to 100.".to_string());
        }

        let strength = temp as f32 / 100.0;
        let result: BOOL = unsafe { msg_send![*self.client, setStrength:strength commit:YES] };

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
