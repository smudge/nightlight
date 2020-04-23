extern crate objc;

use objc::rc::StrongPtr;
use objc::runtime::{Object, BOOL, YES};
use objc::{class, msg_send, sel, sel_impl};
use std::os::raw::c_int;

#[link(name = "CoreBrightness", kind = "framework")]
extern "C" {}

pub struct NightShift {
    client: StrongPtr,
}

#[derive(Default)]
#[repr(C)]
pub struct Time {
    hour: c_int,
    minute: c_int,
}

#[derive(Default)]
#[repr(C)]
pub struct Schedule {
    from_time: Time,
    to_time: Time,
}

#[derive(Default)]
#[repr(C)]
pub struct Status {
    active: BOOL,
    enabled: BOOL,
    sun_schedule_permitted: BOOL,
    mode: c_int,
    schedule: Schedule,
    disable_flags: u64,
    available: BOOL,
}

impl Status {
    pub fn active(&self) -> bool {
        self.active == (true as BOOL)
    }

    pub fn enabled(&self) -> bool {
        self.enabled == (true as BOOL)
    }

    pub fn sun_schedule_permitted(&self) -> bool {
        self.sun_schedule_permitted == (true as BOOL)
    }

    pub fn mode(&self) -> i32 {
        self.mode as i32
    }

    pub fn disable_flags(&self) -> u64 {
        self.disable_flags
    }

    pub fn available(&self) -> bool {
        self.available == (true as BOOL)
    }

    pub fn from_time(&self) -> String {
        format!(
            "{}:{}",
            self.schedule.from_time.hour, self.schedule.from_time.minute
        )
    }

    pub fn to_time(&self) -> String {
        format!(
            "{}:{}",
            self.schedule.to_time.hour, self.schedule.to_time.minute
        )
    }
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

    pub fn status(&self) -> Result<Status, String> {
        let mut status = Status::default();
        let result: BOOL = unsafe { msg_send![*self.client, getBlueLightStatus: &mut status] };
        if result == (true as BOOL) {
            Ok(status)
        } else {
            Err("Failed to get status".to_string())
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
