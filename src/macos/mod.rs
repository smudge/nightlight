extern crate objc;

#[link(name = "CoreBrightness", kind = "framework")]
unsafe extern "C" {}

mod client;
mod locale;
mod status;

pub use self::client::Client;
pub use self::locale::Locale;
pub use self::status::BlueLightStatus;
