use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Dev,
    Build
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// impl From<Option<Command> for Mode {
//     assert!(cli.command.is_some()); // programmer error, UI should enforce
//     let mode = match cli.command {
//         Some(Command::Dev) => Mode::Dev,
//         _ => Mode::Build
//     };
// }

