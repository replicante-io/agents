use std::fmt;

use replicante_agent::Error;
use replicante_agent::ErrorKind as BaseKind;

/// Zookeeper specifc error kinds.
#[derive(Debug)]
pub enum ErrorKind {
    /// Alias for `ConfigLoad`.
    ConfigLoad,

    /// Alias for `ConfigOption`.
    ConfigOption(&'static str),

    /// Alias for `Initialisation`.
    Initialisation(String),

    /// Alias for `Io`.
    Io(String),

    /// Alias for `StoreOpFailed`.
    StoreOpFailed(&'static str),

    /// Version information could not be parsd.
    VersionParse,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Zookeeper Error")
    }
}

impl From<ErrorKind> for BaseKind {
    fn from(error: ErrorKind) -> BaseKind {
        match error {
            ErrorKind::ConfigLoad => BaseKind::ConfigLoad,
            ErrorKind::ConfigOption(option) => BaseKind::ConfigOption(option),
            ErrorKind::Initialisation(message) => BaseKind::Initialisation(message),
            ErrorKind::Io(path) => BaseKind::Io(path),
            ErrorKind::StoreOpFailed(op) => BaseKind::StoreOpFailed(op),
            ErrorKind::VersionParse => BaseKind::ResponseDecode("text", "version"),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(error: ErrorKind) -> Error {
        Error::from(BaseKind::from(error))
    }
}
