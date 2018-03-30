use iron::prelude::*;
use iron::Handler;
use iron::status;

use iron_json_response::JsonResponse;
use iron_json_response::JsonResponseMiddleware;

use opentracingrust::utils::FailSpan;

use replicante_agent_models::NodeStatus;

use super::super::AgentContainer;
use super::super::error::otr_to_iron;
use super::super::util::tracing::HeadersCarrier;


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
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let mut span = HeadersCarrier::child_of("status", &mut request.headers, self.agent.tracer())
            .map_err(otr_to_iron)?.auto_finish();
        let shards = self.agent.shards(&mut span).fail_span(&mut span)?;
        let status = NodeStatus::new(shards);
        let mut response = Response::new();
        match HeadersCarrier::inject(span.context(), &mut response.headers, self.agent.tracer()) {
            Ok(_) => (),
            Err(err) => {
                // TODO: convert to logging.
                println!("Failed to inject span: {:?}", err)
            }
        };
        response.set_mut(JsonResponse::json(&status)).set_mut(status::Ok);
        Ok(response)
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use iron::Headers;
    use iron::IronError;
    use iron_test::request;
    use iron_test::response;

    use replicante_agent_models::Shard;
    use replicante_agent_models::ShardRole;

    use super::StatusHandler;
    use super::super::super::Agent;
    use super::super::super::testing::MockAgent;

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
        let (mut agent, _receiver) = MockAgent::new();
        agent.shards = Ok(vec![Shard::new("test-shard", ShardRole::Primary, Some(1), 2)]);
        let result = request_get(Box::new(agent)).unwrap();
        assert_eq!(result, r#"{"shards":[{"id":"test-shard","role":"Primary","lag":1,"last_op":2}]}"#);
    }
}
