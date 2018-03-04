use iron::prelude::*;
use iron::Handler;
use iron::status;

use iron_json_response::JsonResponse;
use iron_json_response::JsonResponseMiddleware;

use opentracingrust::utils::FailSpan;

use super::super::AgentContainer;
use super::super::models::Shard;
use super::super::util::tracing::ResponseCarrier;


/// Handler implementing the /api/v1/status endpoint.
pub struct StatusHandler {
    agent: AgentContainer
}

impl StatusHandler {
    pub fn new(agent: AgentContainer) -> Chain {
        let handler = StatusHandler { agent };
        let mut chain = Chain::new(handler);
        chain.link_after(JsonResponseMiddleware::new());
        chain
    }
}

impl Handler for StatusHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let mut span = self.agent.tracer().span("status").auto_finish();
        let shards = StatusRespone {
            shards: self.agent.shards(&mut span).fail_span(&mut span)?
        };
        let mut response = Response::new();
        match ResponseCarrier::inject(span.context(), &mut response, self.agent.tracer()) {
            Ok(_) => (),
            Err(err) => {
                // TODO: convert to logging.
                println!("Failed to inject span: {:?}", err)
            }
        };
        response.set_mut(JsonResponse::json(&shards)).set_mut(status::Ok);
        Ok(response)
    }
}


/// Wrapps the shards info for API response.
#[derive(Serialize)]
struct StatusRespone {
    shards: Vec<Shard>
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use iron::Headers;
    use iron::IronError;
    use iron_test::request;
    use iron_test::response;

    use opentracingrust::Span;
    use opentracingrust::Tracer;
    use opentracingrust::tracers::NoopTracer;

    use super::StatusHandler;
    use super::super::super::Agent;
    use super::super::super::AgentError;
    use super::super::super::AgentResult;

    use super::super::super::models::AgentVersion;
    use super::super::super::models::DatastoreVersion;
    use super::super::super::models::Shard;
    use super::super::super::models::ShardRole;

    struct TestAgent {
        tracer: Tracer,
    }

    impl Agent for TestAgent {
        fn agent_version(&self, _: &mut Span) -> AgentResult<AgentVersion> {
            Ok(AgentVersion::new("dcd", "1.2.3", "tainted"))
        }

        fn datastore_version(&self, _: &mut Span) -> AgentResult<DatastoreVersion> {
            Err(AgentError::GenericError(String::from("Not Needed")))
        }

        fn tracer(&self) -> &Tracer {
            &self.tracer
        }

        fn shards(&self, _: &mut Span) -> AgentResult<Vec<Shard>> {
            Ok(vec![
               Shard::new("test-shard", ShardRole::Primary, Some(1), 2)
            ])
        }
    }

    fn request_get(agent: Box<Agent>) -> Result<String, IronError> {
        let handler = StatusHandler::new(Arc::new(agent));
        request::get(
            "http://localhost:3000/api/v1/status",
            Headers::new(), &handler
        )
        .map(|response| {
            let body = response::extract_body_to_bytes(response);
            String::from_utf8(body).unwrap()
        })
    }

    #[test]
    fn status_retruns_shards() {
        let (tracer, _receiver) = NoopTracer::new();
        let result = request_get(Box::new(TestAgent { tracer })).unwrap();
        assert_eq!(result, r#"{"shards":[{"id":"test-shard","role":"Primary","lag":1,"last_op":2}]}"#);
    }
}
