use std::os::raw::{c_float, c_int};

mod locale;

pub use self::locale::Locale;

pub struct CBBlueLightClient {}

impl CBBlueLightClient {
    pub fn new() -> CBBlueLightClient {
        CBBlueLightClient {}
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        Ok(())
    }

    pub fn set_mode(&self, mode: c_int) -> Result<(), String> {
        Ok(())
    }

    pub fn set_schedule(&self, from: (u8, u8), to: (u8, u8)) -> Result<(), String> {
        Ok(())
    }

    pub fn set_strength(&self, strength: c_float) -> Result<(), String> {
        Ok(())
    }

    pub fn get_strength(&self) -> Result<i32, String> {
        Ok(100)
    }

    pub fn get_enabled(&self) -> Result<bool, String> {
        Ok(false)
    }

    pub fn get_mode(&self) -> Result<i32, String> {
        Ok(0)
    }

    pub fn get_schedule(&self) -> Result<((u8, u8), (u8, u8)), String> {
        Ok(((0, 0), (1, 1)))
    }
}
