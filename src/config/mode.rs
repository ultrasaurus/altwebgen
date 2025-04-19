use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Dev,
    Build
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Mode::Dev => "Mode::Dev",
            Mode::Build => "Mode::Build"
        };
        write!(f, "{}", s)
    }
}

// impl From<Option<Command> for Mode {
//     assert!(cli.command.is_some()); // programmer error, UI should enforce
//     let mode = match cli.command {
//         Some(Command::Dev) => Mode::Dev,
//         _ => Mode::Build
//     };
// }

