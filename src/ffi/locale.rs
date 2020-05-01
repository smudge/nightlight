extern crate objc_foundation;

use self::objc_foundation::{INSString, NSString};
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct Locale {
    is_24_hr: bool,
}

thread_local! {
    static CURRENT_LOCALE: RwLock<Arc<Locale>> = RwLock::new(Arc::new(Locale::default()));
}

impl Locale {
    pub fn initialize() -> Result<(), String> {
        let current_locale = Locale::get_current()?;
        CURRENT_LOCALE.with(|c| *c.write().unwrap() = Arc::new(current_locale));
        Ok(())
    }

    pub fn current() -> Arc<Locale> {
        CURRENT_LOCALE.with(|c| c.read().unwrap().clone())
    }

    pub fn is_24_hr(&self) -> bool {
        self.is_24_hr
    }

    fn get_current() -> Result<Locale, String> {
        let nslocale = class!(NSLocale);
        let nsdateformatter = class!(NSDateFormatter);
        let j = NSString::from_str("j");

        // Get the current locale:
        let locale: *mut Object = unsafe { msg_send![nslocale, currentLocale] };

        // Get the system to tell what time format is used for this locale:
        let format: *const NSString =
            unsafe { msg_send![nsdateformatter, dateFormatFromTemplate:j options:0 locale:locale] };

        match unsafe { format.as_ref() } {
            Some(f) => Ok(Locale {
                is_24_hr: !f.as_str().contains("a"),
            }),
            None => Err("Unable to determine system locale!".to_string()),
        }
    }
}
