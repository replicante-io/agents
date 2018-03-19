use iron::prelude::*;
use iron::Handler;
use iron::status;

use iron_json_response::JsonResponse;
use iron_json_response::JsonResponseMiddleware;

use opentracingrust::utils::FailSpan;

use super::super::AgentContainer;
use super::super::error::otr_to_iron;

use super::super::models::AgentInfo;
use super::super::models::DatastoreInfo;
use super::super::util::tracing::HeadersCarrier;


/// Handler implementing the /api/v1/info endpoint.
pub struct InfoHandler {
    agent: AgentContainer,
}

impl InfoHandler {
    pub fn new(agent: AgentContainer) -> Chain {
        let handler = InfoHandler { agent };
        let mut chain = Chain::new(handler);
        chain.link_after(JsonResponseMiddleware::new());
        chain
    }
}

impl Handler for InfoHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let mut span = HeadersCarrier::child_of("info", &mut request.headers, self.agent.tracer())
            .map_err(otr_to_iron)?.auto_finish();

        let agent_version = self.agent.agent_version(&mut span).fail_span(&mut span)?;
        let agent = AgentInfo::new(agent_version);
        let datastore = self.agent.datastore_info(&mut span).fail_span(&mut span)?;
        let info = InfoResponse { datastore, agent };

        let mut response = Response::new();
        match HeadersCarrier::inject(span.context(), &mut response.headers, self.agent.tracer()) {
            Ok(_) => (),
            Err(err) => {
                // TODO: convert to logging.
                println!("Failed to inject span: {:?}", err)
            }
        };
        response.set_mut(JsonResponse::json(info)).set_mut(status::Ok);
        Ok(response)
    }
}


/// Wrapps the agent and datastore versions for API response.
#[derive(Serialize)]
struct InfoResponse {
    agent: AgentInfo,
    datastore: DatastoreInfo,
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use iron::IronError;
    use iron::Headers;
    use iron_test::request;
    use iron_test::response;

    use super::InfoHandler;
    use super::super::super::Agent;
    use super::super::super::AgentError;

    use super::super::super::testing::MockAgent;

    fn request_get(agent: Box<Agent>) -> Result<String, IronError> {
        let handler = InfoHandler::new(Arc::new(agent));
        request::get(
            "http://localhost:3000/api/v1/index",
            Headers::new(), &handler
        )
        .map(|response| {
            let body = response::extract_body_to_bytes(response);
            String::from_utf8(body).unwrap()
        })
    }

    #[test]
    fn info_handler_returns_error() {
        let (mut agent, _receiver) = MockAgent::new();
        agent.datastore_info = Err(AgentError::GenericError(String::from("Testing failure")));
        let result = request_get(Box::new(agent));
        assert!(result.is_err());
        if let Some(result) = result.err() {
            let body = response::extract_body_to_bytes(result.response);
            let body = String::from_utf8(body).unwrap();
            assert_eq!(body, r#"{"error":"Generic error: Testing failure","kind":"GenericError"}"#);
        }
    }

    #[test]
    fn info_handler_returns_version() {
        let (agent, _receiver) = MockAgent::new();
        let result = request_get(Box::new(agent)).unwrap();
        let expected = r#"{"agent":{"version":{"checkout":"dcd","number":"1.2.3","taint":"tainted"}},"datastore":{"kind":"DB","name":"mock","version":"1.2.3"}}"#;
        assert_eq!(result, expected);
    }
}
