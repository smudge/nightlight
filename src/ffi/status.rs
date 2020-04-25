use objc::runtime::{BOOL, YES};
use std::os::raw::c_int;

mod padding;

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
pub struct InnerStatus {
    _active: BOOL,
    enabled: BOOL,
    _sun_schedule_permitted: BOOL,
    mode: c_int,
    schedule: Schedule,
    _disable_flags: u64,
    _available: BOOL,
    padding: padding::Padding,
}

#[derive(Default)]
pub struct BlueLightStatus {
    inner: InnerStatus,
}

impl BlueLightStatus {
    pub fn c_ptr() -> InnerStatus {
        InnerStatus::default()
    }

    pub fn new(inner: InnerStatus) -> BlueLightStatus {
        if !inner.padding.is_empty() {
            eprintln!(
                "======== \
                \nWarning: Your version of macOS may be new, resulting in unexpected behavior. \
                \n========"
            )
        }
        BlueLightStatus { inner }
    }

    pub fn enabled(&self) -> bool {
        self.inner.enabled == YES
    }

    pub fn mode(&self) -> i32 {
        self.inner.mode as i32
    }

    pub fn from_time(&self) -> (u8, u8) {
        (
            self.inner.schedule.from_time.hour as u8,
            self.inner.schedule.from_time.minute as u8,
        )
    }

    pub fn to_time(&self) -> (u8, u8) {
        (
            self.inner.schedule.to_time.hour as u8,
            self.inner.schedule.to_time.minute as u8,
        )
    }
}
