use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Transcript {
    Off = 0,
    On,
    Static
}

impl fmt::Display for Transcript {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Transcript::Off => "Transcript::Off",
            Transcript::On => "Transcript::On",
            Transcript::Static => "Transcript::Static",
        };
        write!(f, "{}", s)
    }
}



