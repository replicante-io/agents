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


/// Handler implementing the /api/v1/status endpoint.
pub struct Shards {
    agent: Arc<Agent>,
    context: AgentContext,
}

impl Shards {
    pub fn make(agent: Arc<Agent>, context: AgentContext) -> Chain {
        let handler = Shards { agent, context };
        let mut chain = Chain::new(handler);
        chain.link_after(JsonResponseMiddleware::new());
        chain
    }
}

impl Handler for Shards {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let tracer = &self.context.tracer;
        let mut span = HeadersCarrier::child_of("status", &mut request.headers, tracer)
            .map_err(otr_to_iron)?.auto_finish();
        let shards = self.agent.shards(&mut span).fail_span(&mut span)?;
        let mut response = Response::new();
        match HeadersCarrier::inject(span.context(), &mut response.headers, tracer) {
            Ok(_) => (),
            Err(error) => {
                let error = format!("{:?}", error);
                error!(self.context.logger, "Failed to inject span"; "error" => error);
            }
        };
        response.set_mut(JsonResponse::json(&shards)).set_mut(status::Ok);
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
    use replicante_agent_models::Shards as ShardsModel;
    use replicante_agent_models::ShardRole;

    use super::super::super::Agent;
    use super::super::super::AgentContext;
    use super::super::super::testing::MockAgent;
    use super::Shards;

    fn request_get<A>(agent: A) -> Result<String, IronError> 
        where A: Agent + 'static
    {
        let (context, extra) = AgentContext::mock();
        let handler = Shards::make(Arc::new(agent), context);
        let response = request::get(
            "http://localhost:3000/api/v1/status",
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
    fn status_retruns_shards() {
        let mut agent = MockAgent::new();
        agent.shards = Ok(ShardsModel::new(vec![
            Shard::new("test-shard", ShardRole::Primary, Some(1), 2)
        ]));
        let result = request_get(agent).unwrap();
        assert_eq!(result, r#"{"shards":[{"id":"test-shard","role":"Primary","lag":1,"last_op":2}]}"#);
    }
}
