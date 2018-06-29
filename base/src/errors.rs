use error_chain::ChainedError;

use iron::IronError;
use iron::headers::ContentType;
use iron::prelude::*;
use iron::status;
use iron_json_response::JsonResponse;

use opentracingrust::Error as OTError;

use serde_json;


error_chain! {}

impl From<Error> for IronError {
    fn from(error: Error) -> Self {
        let wrapper = JsonErrorWrapper { error: error.display_chain().to_string() };
        let mut response = Response::with(
            (status::InternalServerError, serde_json::to_string(&wrapper).unwrap())
        );
        response.headers.set(ContentType::json());
        let error = Box::new(error);
        IronError { error, response }
    }
}


/// Wrapps an Error into a serializable struct.
///
/// This struct is filled in and used by the conversion
/// of an Error to an IronError.
#[derive(Serialize)]
struct AgentErrorResponse {
    error: String,
    kind: String,
}


/// TODO
#[derive(Serialize)]
struct JsonErrorWrapper {
    error: String,
}


/// Conver and OpenTracingRust error into an IronError.
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
pub fn otr_to_iron(error: OTError) -> IronError {
    // TODO: OTError should really have implemented `Error` :-(
    let payload = AgentErrorResponse {
        error: format!("{:?}", error),
        kind: "OpenTracingRustError".into()
    };
    let error: Error = ErrorKind::Msg("OpenTracingRust Error".into()).into();
    let mut response = Response::new();
    response.set_mut(JsonResponse::json(payload)).set_mut(status::BadRequest);
    IronError {
        error: Box::new(error),
        response
    }
}


#[cfg(test)]
mod tests {
    use iron::IronResult;
    use iron::Headers;
    use iron::Response;
    use iron::Request;
    use iron::headers::ContentType;

    use iron_test::request;
    use iron_test::response;

    use super::Result;
    use super::ResultExt;

    fn failing(_: &mut Request) -> IronResult<Response> {
        let err: Result<Response> = Err("test".into());
        Ok(err.chain_err(|| "chained").chain_err(|| "failures")?)
    }

    #[test]
    fn error_conversion() {
        let response = request::get("http://host:16016/", Headers::new(), &failing);
        let response = match response {
            Err(error) => error.response,
            Ok(_) => panic!("Request should fail")
        };

        let content_type = response.headers.get::<ContentType>().unwrap().clone();
        assert_eq!(content_type, ContentType::json());

        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(result_body, r#"{"error":"Error: failures\nCaused by: chained\nCaused by: test\n"}"#);
    }
}
