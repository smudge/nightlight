use objc::rc::StrongPtr;
use objc::runtime::{Object, BOOL, YES};
use objc::{class, msg_send, sel, sel_impl};
use std::os::raw::c_int;

#[link(name = "CoreBrightness", kind = "framework")]
extern "C" {}

#[derive(Default)]
#[repr(C)]
struct Time {
    hour: c_int,
    minute: c_int,
}

#[derive(Default)]
#[repr(C)]
struct Schedule {
    from_time: Time,
    to_time: Time,
}

#[derive(Default)]
#[repr(C)]
struct InnerStatus {
    active: BOOL,
    enabled: BOOL,
    sun_schedule_permitted: BOOL,
    mode: c_int,
    schedule: Schedule,
    disable_flags: u64,
    available: BOOL,
}

#[derive(Default)]
pub struct Status {
    inner: InnerStatus,
}

pub struct Client {
    inner: StrongPtr,
}

impl Client {
    pub fn new() -> Client {
        let client_class = class!(CBBlueLightClient);
        let client = unsafe {
            let obj: *mut Object = msg_send![client_class, alloc];
            let obj: *mut Object = msg_send![obj, init];
            StrongPtr::new(obj)
        };
        Client { inner: client }
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        let result: BOOL = unsafe { msg_send![*self.inner, setEnabled: (enabled as BOOL)] };
        if result == (true as BOOL) {
            Ok(())
        } else {
            Err(format!("Failed to turn Night Shift {}", on_or_off(enabled)))
        }
    }

    pub fn set_strength(&self, strength: f32) -> Result<(), String> {
        let result: BOOL = unsafe { msg_send![*self.inner, setStrength:strength commit:YES] };

        if result == (true as BOOL) {
            Ok(())
        } else {
            Err("Failed to set color temperature".to_string())
        }
    }

    pub fn status(&self) -> Result<Status, String> {
        let mut status = InnerStatus::default();
        let result: BOOL = unsafe { msg_send![*self.inner, getBlueLightStatus: &mut status] };
        if result == (true as BOOL) {
            Ok(Status { inner: status })
        } else {
            Err("Failed to get status".to_string())
        }
    }
}

impl Status {
    pub fn active(&self) -> bool {
        self.inner.active == (true as BOOL)
    }

    pub fn enabled(&self) -> bool {
        self.inner.enabled == (true as BOOL)
    }

    pub fn sun_schedule_permitted(&self) -> bool {
        self.inner.sun_schedule_permitted == (true as BOOL)
    }

    pub fn mode(&self) -> i32 {
        self.inner.mode as i32
    }

    pub fn disable_flags(&self) -> u64 {
        self.inner.disable_flags
    }

    pub fn available(&self) -> bool {
        self.inner.available == (true as BOOL)
    }

    pub fn from_time(&self) -> String {
        format!(
            "{}:{}",
            self.inner.schedule.from_time.hour, self.inner.schedule.from_time.minute
        )
    }

    pub fn to_time(&self) -> String {
        format!(
            "{}:{}",
            self.inner.schedule.to_time.hour, self.inner.schedule.to_time.minute
        )
    }
}

fn on_or_off(value: bool) -> String {
    if value {
        "on".to_string()
    } else {
        "off".to_string()
    }
}
