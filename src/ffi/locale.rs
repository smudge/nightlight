extern crate objc_foundation;

use self::objc_foundation::{INSString, NSString};
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};

pub struct Locale {
    is_24_hr: bool,
}

impl Locale {
    pub fn current() -> Result<Locale, String> {
        let nslocale = class!(NSLocale);
        let nsdateformatter = class!(NSDateFormatter);
        let j = NSString::from_str("j");

        // Get the current locale:
        let locale: *mut Object = unsafe { msg_send![nslocale, currentLocale] };

        // Get the system to tell what time format is used for this locale:
        let format: *const NSString =
            unsafe { msg_send![nsdateformatter, dateFormatFromTemplate:j options:0 locale:locale] };
        let format = match unsafe { format.as_ref() } {
            Some(s) => s.as_str(),
            None => "ERROR",
        };

        Ok(Locale {
            is_24_hr: !format.contains("a"),
        })
    }

    pub fn is_24_hr(&self) -> bool {
        self.is_24_hr
    }
}
