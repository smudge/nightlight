extern crate objc;

mod ffi;

pub struct NightShift {
    client: ffi::Client,
}

impl NightShift {
    pub fn new() -> NightShift {
        NightShift {
            client: ffi::Client::new(),
        }
    }

    pub fn on(&self) -> Result<(), String> {
        self.toggle(true)
    }

    pub fn off(&self) -> Result<(), String> {
        self.toggle(false)
    }

    pub fn toggle(&self, on: bool) -> Result<(), String> {
        self.client.set_enabled(on)
    }

    pub fn set_temp(&self, temp: i32) -> Result<(), String> {
        if temp < 0 || temp > 100 {
            return Err("Color temperature must be a number from 0 to 100.".to_string());
        }

        self.client.set_strength(temp as f32 / 100.0)
    }

    pub fn is_on(&self) -> Result<bool, String> {
        Ok(self.client.status()?.active())
    }

    pub fn is_enabled(&self) -> Result<bool, String> {
        Ok(self.client.status()?.enabled())
    }
}
