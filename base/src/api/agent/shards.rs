use std::sync::Arc;

use iron::prelude::*;
use iron::status;
use iron::Handler;
use iron_json_response::JsonResponse;
use opentracingrust::Log;

use replicante_util_iron::request_span;
use replicante_util_tracing::fail_span;

use super::super::super::Agent;

/// Handler implementing the /api/v1/status endpoint.
pub struct Shards {
    agent: Arc<dyn Agent>,
}

impl Shards {
    pub fn make(agent: Arc<dyn Agent>) -> Shards {
        Shards { agent }
    }
}

impl Handler for Shards {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let mut span = request_span(request);
        span.log(Log::new().log("span.kind", "server-receive"));
        let shards = self
            .agent
            .shards(&mut span)
            .map_err(|error| fail_span(error, &mut span))?;
        let mut response = Response::new();
        response
            .set_mut(JsonResponse::json(&shards))
            .set_mut(status::Ok);
        span.log(Log::new().log("span.kind", "server-send"));
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

    use replicante_models_agent::CommitOffset;
    use replicante_models_agent::Shard;
    use replicante_models_agent::ShardRole;
    use replicante_models_agent::Shards as ShardsModel;
    use replicante_util_iron::mock_request_span;

    use super::super::super::super::testing::MockAgent;
    use super::super::super::super::Agent;
    use super::super::super::super::AgentContext;
    use super::super::Shards;

    fn request_get<A>(agent: A) -> Result<String, IronError>
    where
        A: Agent + 'static,
    {
        let handler = Shards::make(Arc::new(agent));
        let handler = {
            let mut chain = Chain::new(handler);
            chain.link_after(JsonResponseMiddleware::new());
            chain
        };
        let handler = mock_request_span(AgentContext::mock().tracer, handler);
        let response = request::get(
            "http://localhost:3000/api/v1/status",
            Headers::new(),
            &handler,
        )
        .map(|response| {
            let body = response::extract_body_to_bytes(response);
            String::from_utf8(body).unwrap()
        });
        drop(handler);
        response
    }

    #[test]
    fn status_retruns_shards() {
        let mut agent = MockAgent::new();
        agent.shards = Ok(ShardsModel::new(vec![Shard::new(
            "test-shard",
            ShardRole::Primary,
            Some(CommitOffset::seconds(2)),
            Some(CommitOffset::seconds(1)),
        )]));
        let result = request_get(agent).unwrap();
        let expected = concat!(
            r#"{"shards":[{"commit_offset":{"unit":"seconds","value":2},"id":"test-shard","#,
            r#""lag":{"unit":"seconds","value":1},"role":"primary"}]}"#
        );
        assert_eq!(result, expected);
    }
}
