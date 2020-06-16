use std::sync::Arc;

pub struct Locale {
    is_24_hr: bool,
}

impl Locale {
    pub fn initialize() -> Result<(), String> {
        Ok(())
    }

    pub fn current() -> Arc<Locale> {
        Arc::new(Locale { is_24_hr: true })
    }

    pub fn is_24_hr(&self) -> bool {
        true
    }
}
