use crate::ffi::BlueLightStatus;
use objc::rc::StrongPtr;
use objc::runtime::{Object, BOOL, YES};
use objc::{class, msg_send, sel, sel_impl};
use std::os::raw::{c_float, c_int};

pub struct CBBlueLightClient {
    inner: StrongPtr,
}

impl CBBlueLightClient {
    pub fn new() -> CBBlueLightClient {
        let client_class = class!(CBBlueLightClient);
        let client = unsafe {
            let obj: *mut Object = msg_send![client_class, alloc];
            let obj: *mut Object = msg_send![obj, init];
            StrongPtr::new(obj)
        };
        CBBlueLightClient { inner: client }
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        let result: BOOL = unsafe { msg_send![*self.inner, setEnabled: (enabled as BOOL)] };
        if result == YES {
            Ok(())
        } else {
            Err(format!("Failed to set enabled to {}", enabled).to_string())
        }
    }

    pub fn set_mode(&self, mode: u8) -> Result<(), String> {
        let result: BOOL = unsafe { msg_send![*self.inner, setMode: mode as c_int] };

        if result == YES {
            Ok(())
        } else {
            Err("Failed to set schedule".to_string())
        }
    }

    pub fn set_schedule(&self, from: (u8, u8), to: (u8, u8)) -> Result<(), String> {
        let ptr = BlueLightStatus::sched_ptr(from, to);
        let result: BOOL = unsafe { msg_send![*self.inner, setSchedule: &ptr] };

        if result == YES {
            Ok(())
        } else {
            Err("Failed to set schedule".to_string())
        }
    }

    pub fn set_strength(&self, strength: c_float) -> Result<(), String> {
        let result: BOOL = unsafe { msg_send![*self.inner, setStrength:strength commit:YES] };

        if result == YES {
            Ok(())
        } else {
            Err("Failed to set color temperature".to_string())
        }
    }

    pub fn get_strength(&self) -> Result<i32, String> {
        let mut value: c_float = -1.0;
        let result: BOOL = unsafe { msg_send![*self.inner, getStrength: &mut value] };

        if result == YES && value >= 0.0 {
            Ok((value * 100.0) as i32)
        } else {
            Err("Failed to get color temperature".to_string())
        }
    }

    pub fn status(&self) -> Result<BlueLightStatus, String> {
        let mut ptr = BlueLightStatus::c_ptr();
        let result: BOOL = unsafe { msg_send![*self.inner, getBlueLightStatus: &mut ptr.as_mut()] };
        if result == YES {
            Ok(BlueLightStatus::new(*ptr))
        } else {
            Err("Failed to get status".to_string())
        }
    }
}
