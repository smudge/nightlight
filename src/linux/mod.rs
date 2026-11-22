use crate::backend::Backend;

mod command;
mod gnome;
mod locale;
mod transaction;

pub use self::locale::Locale;

pub(crate) fn backend() -> Box<dyn Backend> {
    Box::new(gnome::Gnome::new())
}
