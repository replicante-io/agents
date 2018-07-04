use std::sync::Arc;

use iron::prelude::*;
use iron::Handler;
use iron::status;

use iron_json_response::JsonResponse;
use iron_json_response::JsonResponseMiddleware;

use opentracingrust::utils::FailSpan;

use super::super::Agent;
use super::super::AgentContext;
use super::super::errors::otr_to_iron;
use super::super::util::tracing::HeadersCarrier;


/// Handler implementing the /api/v1/info/agent endpoint.
pub struct AgentInfo {
    agent: Arc<Agent>,
    context: AgentContext,
}

impl AgentInfo {
    pub fn make(agent: Arc<Agent>, context: AgentContext) -> Chain {
        let handler = AgentInfo { agent, context };
        let mut chain = Chain::new(handler);
        chain.link_after(JsonResponseMiddleware::new());
        chain
    }
}

impl Handler for AgentInfo {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let tracer = &self.context.tracer;
        let mut span = HeadersCarrier::child_of("agent-info", &mut request.headers, tracer)
            .map_err(otr_to_iron)?.auto_finish();

        let info = self.agent.agent_info(&mut span).fail_span(&mut span)?;
        let mut response = Response::new();
        match HeadersCarrier::inject(span.context(), &mut response.headers, tracer) {
            Ok(_) => (),
            Err(error) => {
                let error = format!("{:?}", error);
                error!(self.context.logger, "Failed to inject span"; "error" => error);
            }
        };
        response.set_mut(JsonResponse::json(info)).set_mut(status::Ok);
        Ok(response)
    }
}


/// Handler implementing the /api/v1/info/datastore endpoint.
pub struct DatastoreInfo {
    agent: Arc<Agent>,
    context: AgentContext,
}

impl DatastoreInfo {
    pub fn make(agent: Arc<Agent>, context: AgentContext) -> Chain {
        let handler = DatastoreInfo { agent, context };
        let mut chain = Chain::new(handler);
        chain.link_after(JsonResponseMiddleware::new());
        chain
    }
}

impl Handler for DatastoreInfo {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let tracer = &self.context.tracer;
        let mut span = HeadersCarrier::child_of("datastore-info", &mut request.headers, tracer)
            .map_err(otr_to_iron)?.auto_finish();
        let info = self.agent.datastore_info(&mut span).fail_span(&mut span)?;
        let mut response = Response::new();
        match HeadersCarrier::inject(span.context(), &mut response.headers, tracer) {
            Ok(_) => (),
            Err(error) => {
                let error = format!("{:?}", error);
                error!(self.context.logger, "Failed to inject span"; "error" => error);
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

        use super::super::super::super::Agent;
        use super::super::super::super::AgentContext;
        use super::super::super::super::testing::MockAgent;
        use super::super::AgentInfo;


        fn get<A>(agent: A) -> Result<String, IronError> 
            where A: Agent + 'static
        {
            let (context, extra) = AgentContext::mock();
            let handler = AgentInfo::make(Arc::new(agent), context);
            let response = request::get(
                "http://localhost:3000/api/v1/info/agent",
                Headers::new(), &handler
            )
            .map(|response| {
                let body = response::extract_body_to_bytes(response);
                String::from_utf8(body).unwrap()
            });
            drop(extra);
            drop(handler);
            response
        }

        #[test]
        fn returns_error() {
            let mut agent = MockAgent::new();
            agent.agent_info = Err("Testing failure".into());
            let result = get(agent);
            assert!(result.is_err());
            if let Some(result) = result.err() {
                let body = response::extract_body_to_bytes(result.response);
                let body = String::from_utf8(body).unwrap();
                assert_eq!(body, r#"{"error":"Error: Testing failure\n"}"#);
            }
        }

        #[test]
        fn returns_version() {
            let agent = MockAgent::new();
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

        use super::super::super::super::Agent;
        use super::super::super::super::AgentContext;
        use super::super::super::super::testing::MockAgent;
        use super::super::DatastoreInfo;


        fn get<A>(agent: A) -> Result<String, IronError> 
            where A: Agent + 'static
        {
            let (context, extra) = AgentContext::mock();
            let handler = DatastoreInfo::make(Arc::new(agent), context);
            let response = request::get(
                "http://localhost:3000/api/v1/info/datastore",
                Headers::new(), &handler
            )
            .map(|response| {
                let body = response::extract_body_to_bytes(response);
                String::from_utf8(body).unwrap()
            });
            drop(extra);
            drop(handler);
            response
        }

        #[test]
        fn returns_error() {
            let mut agent = MockAgent::new();
            agent.datastore_info = Err("Testing failure".into());
            let result = get(agent);
            assert!(result.is_err());
            if let Some(result) = result.err() {
                let body = response::extract_body_to_bytes(result.response);
                let body = String::from_utf8(body).unwrap();
                assert_eq!(body, r#"{"error":"Error: Testing failure\n"}"#);
            }
        }

        #[test]
        fn returns_version() {
            let agent = MockAgent::new();
            let result = get(agent).unwrap();
            let expected = r#"{"cluster":"cluster","kind":"DB","name":"mock","version":"1.2.3"}"#;
            assert_eq!(result, expected);
        }
    }
}
