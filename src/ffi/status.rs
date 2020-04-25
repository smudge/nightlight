use objc::runtime::{BOOL, YES};
use std::os::raw::c_int;

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
    active: BOOL,
    enabled: BOOL,
    sun_schedule_permitted: BOOL,
    mode: c_int,
    schedule: Schedule,
    disable_flags: u64,
    available: BOOL,
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
        BlueLightStatus { inner }
    }

    pub fn active(&self) -> bool {
        self.inner.active == YES
    }

    pub fn enabled(&self) -> bool {
        self.inner.enabled == YES
    }

    pub fn sun_schedule_permitted(&self) -> bool {
        self.inner.sun_schedule_permitted == YES
    }

    pub fn mode(&self) -> i32 {
        self.inner.mode as i32
    }

    pub fn disable_flags(&self) -> u64 {
        self.inner.disable_flags
    }

    pub fn available(&self) -> bool {
        self.inner.available == YES
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
