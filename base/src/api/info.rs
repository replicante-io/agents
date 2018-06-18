use iron::prelude::*;
use iron::Handler;
use iron::status;

use iron_json_response::JsonResponse;
use iron_json_response::JsonResponseMiddleware;

use opentracingrust::utils::FailSpan;

use super::super::error::otr_to_iron;
use super::super::runner::AgentContainer;
use super::super::util::tracing::HeadersCarrier;


/// Handler implementing the /api/v1/info/agent endpoint.
pub struct AgentInfo {
    agent: AgentContainer,
}

impl AgentInfo {
    pub fn make(agent: AgentContainer) -> Chain {
        let handler = AgentInfo { agent };
        let mut chain = Chain::new(handler);
        chain.link_after(JsonResponseMiddleware::new());
        chain
    }
}

impl Handler for AgentInfo {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let mut span = HeadersCarrier::child_of(
            "agent-info", &mut request.headers, self.agent.tracer()
        ).map_err(otr_to_iron)?.auto_finish();

        let info = self.agent.agent_info(&mut span).fail_span(&mut span)?;
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


/// Handler implementing the /api/v1/info/datastore endpoint.
pub struct DatastoreInfo {
    agent: AgentContainer,
}

impl DatastoreInfo {
    pub fn make(agent: AgentContainer) -> Chain {
        let handler = DatastoreInfo { agent };
        let mut chain = Chain::new(handler);
        chain.link_after(JsonResponseMiddleware::new());
        chain
    }
}

impl Handler for DatastoreInfo {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let mut span = HeadersCarrier::child_of(
            "datastore-info", &mut request.headers, self.agent.tracer()
        ).map_err(otr_to_iron)?.auto_finish();

        let info = self.agent.datastore_info(&mut span).fail_span(&mut span)?;
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


#[cfg(test)]
mod tests {
    mod agent {
        use std::sync::Arc;

        use iron::IronError;
        use iron::Headers;
        use iron_test::request;
        use iron_test::response;

        use super::super::AgentInfo;
        use super::super::super::super::Agent;
        use super::super::super::super::AgentError;

        use super::super::super::super::testing::MockAgent;


        fn get<A>(agent: A) -> Result<String, IronError> 
            where A: Agent + 'static
        {
            let handler = AgentInfo::make(Arc::new(agent));
            request::get(
                "http://localhost:3000/api/v1/info/agent",
                Headers::new(), &handler
            )
            .map(|response| {
                let body = response::extract_body_to_bytes(response);
                String::from_utf8(body).unwrap()
            })
        }

        #[test]
        fn returns_error() {
            let (mut agent, _receiver) = MockAgent::new();
            agent.agent_info = Err(AgentError::GenericError(String::from("Testing failure")));
            let result = get(agent);
            assert!(result.is_err());
            if let Some(result) = result.err() {
                let body = response::extract_body_to_bytes(result.response);
                let body = String::from_utf8(body).unwrap();
                assert_eq!(body, r#"{"error":"Generic error: Testing failure","kind":"GenericError"}"#);
            }
        }

        #[test]
        fn returns_version() {
            let (agent, _receiver) = MockAgent::new();
            let result = get(agent).unwrap();
            let expected = r#"{"version":{"checkout":"dcd","number":"1.2.3","taint":"tainted"}}"#;
            assert_eq!(result, expected);
        }
    }

    mod datastore {
        use std::sync::Arc;

        use iron::IronError;
        use iron::Headers;
        use iron_test::request;
        use iron_test::response;

        use super::super::DatastoreInfo;
        use super::super::super::super::Agent;
        use super::super::super::super::AgentError;

        use super::super::super::super::testing::MockAgent;


        fn get<A>(agent: A) -> Result<String, IronError> 
            where A: Agent + 'static
        {
            let handler = DatastoreInfo::make(Arc::new(agent));
            request::get(
                "http://localhost:3000/api/v1/info/datastore",
                Headers::new(), &handler
            )
            .map(|response| {
                let body = response::extract_body_to_bytes(response);
                String::from_utf8(body).unwrap()
            })
        }

        #[test]
        fn returns_error() {
            let (mut agent, _receiver) = MockAgent::new();
            agent.datastore_info = Err(AgentError::GenericError(String::from("Testing failure")));
            let result = get(agent);
            assert!(result.is_err());
            if let Some(result) = result.err() {
                let body = response::extract_body_to_bytes(result.response);
                let body = String::from_utf8(body).unwrap();
                assert_eq!(body, r#"{"error":"Generic error: Testing failure","kind":"GenericError"}"#);
            }
        }

        #[test]
        fn returns_version() {
            let (agent, _receiver) = MockAgent::new();
            let result = get(agent).unwrap();
            let expected = r#"{"cluster":"cluster","kind":"DB","name":"mock","version":"1.2.3"}"#;
            assert_eq!(result, expected);
        }
    }
}
