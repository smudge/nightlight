extern crate objc;

#[link(name = "CoreBrightness", kind = "framework")]
extern "C" {}

mod client;
mod locale;
mod status;

pub use self::client::CBBlueLightClient;
pub use self::locale::Locale;
pub use self::status::BlueLightStatus;
