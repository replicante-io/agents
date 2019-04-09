use std::sync::Arc;

use iron::prelude::*;
use iron::Handler;
use iron::status;

use iron_json_response::JsonResponse;

use opentracingrust::Log;

use super::super::super::error::fail_span;
use super::super::super::error::otr_to_iron;
use super::super::super::util::tracing::HeadersCarrier;
use super::super::super::Agent;
use super::super::super::AgentContext;


/// Handler implementing the /api/v1/status endpoint.
pub struct Shards {
    agent: Arc<Agent>,
    context: AgentContext,
}

impl Shards {
    pub fn make(agent: Arc<Agent>, context: AgentContext) -> Shards {
        Shards { agent, context }
    }
}

impl Handler for Shards {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let tracer = &self.context.tracer;
        let mut span = HeadersCarrier::child_of("status", &mut request.headers, tracer)
            .map_err(otr_to_iron)?.auto_finish();

        span.log(Log::new().log("span.kind", "server-receive"));
        let shards = self.agent.shards(&mut span).map_err(|error| fail_span(error, &mut span))?;
        span.log(Log::new().log("span.kind", "server-send"));

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

    use iron::Chain;
    use iron::Headers;
    use iron::IronError;
    use iron_json_response::JsonResponseMiddleware;
    use iron_test::request;
    use iron_test::response;

    use replicante_agent_models::CommitOffset;
    use replicante_agent_models::Shard;
    use replicante_agent_models::Shards as ShardsModel;
    use replicante_agent_models::ShardRole;

    use super::super::super::super::Agent;
    use super::super::super::super::AgentContext;
    use super::super::super::super::testing::MockAgent;
    use super::super::Shards;

    fn request_get<A>(agent: A) -> Result<String, IronError> 
        where A: Agent + 'static
    {
        let (context, extra) = AgentContext::mock();
        let handler = Shards::make(Arc::new(agent), context);
        let handler = {
            let mut chain = Chain::new(handler);
            chain.link_after(JsonResponseMiddleware::new());
            chain
        };
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
            Shard::new(
                "test-shard", ShardRole::Primary, Some(CommitOffset::seconds(2)),
                Some(CommitOffset::seconds(1))
            )
        ]));
        let result = request_get(agent).unwrap();
        let expected = concat!(
            r#"{"shards":[{"commit_offset":{"unit":"seconds","value":2},"id":"test-shard","#,
            r#""lag":{"unit":"seconds","value":1},"role":"primary"}]}"#
        );
        assert_eq!(result, expected);
    }
}
