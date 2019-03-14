use std::fmt;

use replicante_agent::Error;
use replicante_agent::ErrorKind as BaseKind;


/// MongoDB specifc error kinds.
#[derive(Debug)]
pub enum ErrorKind {
    /// BSON specifc `ResponseDecode`.
    BsonDecode(&'static str),

    /// Alias for `ConfigLoad`.
    ConfigLoad,

    /// Alias for `ConfigOption`.
    ConfigOption(&'static str),

    /// Alias for `Connection`.
    Connection(&'static str, String),

    /// Alias for `Initialisation`.
    Initialisation(String),

    /// Alias for `IO`.
    Io(String),

    /// `InvalidStoreState` caused by the inability to find a primary.
    MembersNoPrimary,

    /// `InvalidStoreState` caused by the inability to find self in the replica set.
    MembersNoSelf,

    /// Alias for `StoreOpFailed`.
    StoreOpFailed(&'static str),

    /// `InvalidStoreState` caused by an unsupported node's myState code.
    UnsupportedSateId(i32),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MongoDB Error")
    }
}

impl From<ErrorKind> for BaseKind {
    fn from(error: ErrorKind) -> BaseKind {
        match error {
            ErrorKind::BsonDecode(operation) => BaseKind::ResponseDecode("bson", operation),
            ErrorKind::ConfigLoad => BaseKind::ConfigLoad,
            ErrorKind::ConfigOption(option) => BaseKind::ConfigOption(option),
            ErrorKind::Connection(system, address) => BaseKind::Connection(system, address),
            ErrorKind::Initialisation(message) => BaseKind::Initialisation(message),
            ErrorKind::Io(path) => BaseKind::Io(path),
            ErrorKind::MembersNoPrimary =>
                BaseKind::InvalidStoreState("primary node not in members list".into()),
            ErrorKind::MembersNoSelf =>
                BaseKind::InvalidStoreState("self not in members list".into()),
            ErrorKind::StoreOpFailed(op) => BaseKind::StoreOpFailed(op),
            ErrorKind::UnsupportedSateId(state) =>
                BaseKind::InvalidStoreState(format!("unsupported node state {}", state)),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(error: ErrorKind) -> Error {
        Error::from(BaseKind::from(error))
    }
}
