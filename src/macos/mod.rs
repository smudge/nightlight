extern crate objc;

#[link(name = "CoreBrightness", kind = "framework")]
unsafe extern "C" {}

mod client;
mod locale;
mod status;

use crate::backend::Backend;
use crate::{Schedule, Status};

pub use self::client::Client;
pub use self::locale::Locale;
pub use self::status::BlueLightStatus;

pub(crate) fn backend() -> Box<dyn Backend> {
    Box::new(MacOS {
        client: Client::new(),
    })
}

pub(crate) fn backend_named(name: &str) -> Result<Box<dyn Backend>, String> {
    match name {
        "macos" => Ok(backend()),
        _ => Err(format!(
            "Unknown night light backend '{}'. Supported backends: macos",
            name
        )),
    }
}

struct MacOS {
    client: Client,
}

impl Backend for MacOS {
    fn name(&self) -> &'static str {
        "macos"
    }

    fn status(&self) -> Result<Status, String> {
        Ok(Status::from_bool(self.client.get_enabled()?))
    }

    fn set_status(&self, status: Status) -> Result<(), String> {
        self.client
            .set_enabled(status.as_bool())
            .map_err(|_| format!("Failed to turn Night Shift {}", status))
    }

    fn schedule(&self) -> Result<Schedule, String> {
        Schedule::from_mode(self.client.get_mode()?, || self.client.get_schedule())
    }

    // Changing the mode resets the on/off state.
    fn set_schedule(&self, schedule: Schedule) -> Result<(), String> {
        let status = self.status()?;

        match schedule {
            Schedule::Off => self.client.set_mode(0)?,
            Schedule::SunsetToSunrise => self.client.set_mode(1)?,
            Schedule::Custom(from, to) => {
                self.client.set_mode(2)?;
                self.client.set_schedule(from.tuple(), to.tuple())?
            }
        }

        self.set_status(status)
    }

    fn strength(&self) -> Result<i32, String> {
        self.client.get_strength()
    }

    fn set_strength(&self, strength: i32) -> Result<(), String> {
        self.client.set_strength(strength as f32 / 100.0)
    }
}
