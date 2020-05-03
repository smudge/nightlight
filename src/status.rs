use std::fmt;

pub enum Status {
    On,
    Off,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::On => write!(f, "on"),
            Status::Off => write!(f, "off"),
        }
    }
}

impl Status {
    pub fn as_bool(&self) -> bool {
        match self {
            Status::On => true,
            Status::Off => false,
        }
    }

    pub fn is_on(&self) -> bool {
        self.as_bool()
    }

    pub fn is_off(&self) -> bool {
        !self.as_bool()
    }
}
