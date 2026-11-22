use std::sync::Arc;

// Times are always shown in 24-hour format on Linux.
pub struct Locale;

impl Locale {
    pub fn initialize() -> Result<(), String> {
        Ok(())
    }

    pub fn current() -> Arc<Locale> {
        Arc::new(Locale)
    }

    pub fn is_24_hr(&self) -> bool {
        true
    }
}
