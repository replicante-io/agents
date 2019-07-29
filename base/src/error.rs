use std::fmt;

use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use failure::Backtrace;
use failure::Context;
use failure::Fail;

use replicante_util_failure::SerializableFail;

/// Error information returned by functions in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }

    fn name(&self) -> Option<&str> {
        self.kind().kind_name()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error(Context::new(kind))
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let info = SerializableFail::from(self);
        let status = self.kind().http_status();
        HttpResponse::build(status).json(info)
    }

    fn render_response(&self) -> HttpResponse {
        self.error_response()
    }
}

// Support conversion from custom ErrorKind to allow agents to define their own kinds that
// can be converted into base agent error kinds and wrapped in an error.
// See the MongoDB agent code for an example of this.
impl<E> From<Context<E>> for Error
where
    E: Into<ErrorKind> + fmt::Display + Sync + Send,
{
    fn from(context: Context<E>) -> Error {
        let context = context.map(Into::into);
        Error(context)
    }
}

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "actions with kind {} are not available", _0)]
    ActionNotAvailable(String),

    #[fail(display = "invalid configuration: {}", _0)]
    ConfigClash(&'static str),

    #[fail(display = "unable to load configuration")]
    ConfigLoad,

    #[fail(display = "invalid configuration for option {}", _0)]
    ConfigOption(&'static str),

    #[fail(display = "connection error to {} with address '{}'", _0, _1)]
    Connection(&'static str, String),

    /// Generic context agents can use if provided contexts are not enough.
    #[fail(display = "{}", _0)]
    FreeForm(String),

    #[fail(display = "agent initialisation error: {}", _0)]
    Initialisation(String),

    #[fail(display = "invalid datastore state: {}", _0)]
    InvalidStoreState(String),

    #[fail(display = "I/O error on file {}", _0)]
    Io(String),

    #[fail(display = "unable to migrate persistent DB")]
    PersistentMigrate,

    #[fail(display = "unable to open persistent DB {}", _0)]
    PersistentOpen(String),

    #[fail(
        display = "could not decode {} response from store for '{}' operation",
        _0, _1
    )]
    ResponseDecode(&'static str, &'static str),

    #[fail(display = "datastore operation '{}' failed", _0)]
    StoreOpFailed(&'static str),

    #[fail(display = "unable to spawn '{}' thread", _0)]
    ThreadSpawn(&'static str),
}

impl ErrorKind {
    fn http_status(&self) -> StatusCode {
        match self {
            ErrorKind::ActionNotAvailable(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::ActionNotAvailable(_) => "ActionNotAvailable",
            ErrorKind::ConfigClash(_) => "ConfigClash",
            ErrorKind::ConfigLoad => "ConfigLoad",
            ErrorKind::ConfigOption(_) => "ConfigOption",
            ErrorKind::Connection(_, _) => "Connection",
            ErrorKind::FreeForm(_) => "FreeForm",
            ErrorKind::Initialisation(_) => "Initialisation",
            ErrorKind::InvalidStoreState(_) => "InvalidStoreState",
            ErrorKind::Io(_) => "Io",
            ErrorKind::PersistentMigrate => "PersistentMigrate",
            ErrorKind::PersistentOpen(_) => "PersistentOpen",
            ErrorKind::ResponseDecode(_, _) => "ResponseDecode",
            ErrorKind::StoreOpFailed(_) => "StoreOpFailed",
            ErrorKind::ThreadSpawn(_) => "ThreadSpawn",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;
