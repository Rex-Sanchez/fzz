use std::{fmt::Display, io, sync::mpsc::RecvError};

#[derive(Debug)]
pub enum Error {
    UnableToDraw { from: &'static str, e: io::Error },
    EventReceiveError(RecvError),
    NoStdin,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnableToDraw { from, e } => f.write_str(&format!("Unable To Draw: {from}\n{e}")),
            Error::EventReceiveError(e) => f.write_str(&format!("Event Receive Error: {e}")),
            Error::NoStdin => f.write_str(&format!("Nothing to do")),
        }
    }
}

impl From<RecvError> for Error {
    fn from(value: RecvError) -> Self {
        Self::EventReceiveError(value)
    }
}
