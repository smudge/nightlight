use objc::runtime::BOOL;
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
