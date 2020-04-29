extern crate objc_foundation;

#[link(name = "CoreBrightness", kind = "framework")]
extern "C" {}

mod client;
mod status;

pub use self::client::CBBlueLightClient;
pub use self::status::BlueLightStatus;

use self::objc_foundation::{INSString, NSString};
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};

pub fn macos_locale<'a>() -> Result<&'a str, String> {
    let nslocale = class!(NSLocale);

    let current_locale: *mut Object = unsafe { msg_send![nslocale, currentLocale] };
    let locale_identifier: *const NSString = unsafe { msg_send![current_locale, localeIdentifier] };
    match unsafe { locale_identifier.as_ref() } {
        Some(s) => Ok(s.as_str()),
        None => Err("Unable to determine system locale!".to_string()),
    }
}
