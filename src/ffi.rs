#[link(name = "CoreBrightness", kind = "framework")]
extern "C" {}

mod client;
mod status;

pub use self::client::CBBlueLightClient;
pub use self::status::BlueLightStatus;
