use std::fmt;

use failure::err_msg;
use failure::Backtrace;
use failure::Context;
use failure::Fail;

use iron::status;
use iron::IronError;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;

use opentracingrust::Error as OTError;
use opentracingrust::Log;
use opentracingrust::Span;

use replicante_util_failure::SerializableFail;
use replicante_util_iron::into_ironerror;

/// Error information returned by functions in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
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

// Support conversion from custom ErrorKind to allow agents to define their own kinds that
// can be converted into base agent error kinds and wrapped in an error.
// See the MongoDB agent code for an example of this.
impl<E> From<Context<E>> for Error
where
    E: Into<ErrorKind> + fmt::Display + Sync + Send,
{
    fn from(context: Context<E>) -> Error {
        let context = context.map(|e| e.into());
        Error(context)
    }
}

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
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

    #[fail(
        display = "could not decode {} response from store for '{}' operation",
        _0, _1
    )]
    ResponseDecode(&'static str, &'static str),

    #[fail(display = "datastore operation '{}' failed", _0)]
    StoreOpFailed(&'static str),
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;

// **********************
// * Compatibility Code *
// **********************
// IronError compatibility code.
impl From<Error> for IronError {
    fn from(error: Error) -> IronError {
        into_ironerror(error)
    }
}

// OpenTracing compatibility code.
/// Re-implement `FailSpan` for `Fail` errors :-(
pub fn fail_span<E: Fail>(error: E, span: &mut Span) -> E {
    span.tag("error", true);
    span.log(
        Log::new()
            .log("event", "error")
            .log("message", error.to_string())
            .log("error.object", format!("{:?}", error)),
    );
    error
}

/// Convert an OpenTracingRust error into an IronError.
#[allow(clippy::needless_pass_by_value)]
pub fn otr_to_iron(error: OTError) -> IronError {
    let error = format!("{:?}", error);
    let wrapper = SerializableFail {
        error: error.clone(),
        layers: vec![error.clone()],
        trace: None,
    };
    let mut response = Response::new();
    response
        .set_mut(JsonResponse::json(wrapper))
        .set_mut(status::BadRequest);
    // OTError should really have implemented `Error` :-(
    let error = err_msg(error).compat();
    IronError {
        error: Box::new(error),
        response,
    }
}

#[cfg(test)]
mod tests {
    use failure::err_msg;
    use failure::Fail;

    use iron::headers::ContentType;
    use iron::Headers;
    use iron::IronResult;
    use iron::Request;
    use iron::Response;

    use iron_test::request;
    use iron_test::response;

    use super::Error;
    use super::ErrorKind;

    fn failing(_: &mut Request) -> IronResult<Response> {
        let error: Error = err_msg("test")
            .context(ErrorKind::FreeForm("chained".into()))
            .context(ErrorKind::FreeForm("failures".into()))
            .into();
        Err(error.into())
    }

    #[test]
    fn error_conversion() {
        let response = request::get("http://host:16016/", Headers::new(), &failing);
        let response = match response {
            Err(error) => error.response,
            Ok(_) => panic!("Request should fail"),
        };
        let content_type = response.headers.get::<ContentType>().unwrap().clone();
        assert_eq!(content_type, ContentType::json());
        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(
            result_body,
            r#"{"error":"failures","layers":["failures","chained","test"],"trace":null}"#
        );
    }
}
