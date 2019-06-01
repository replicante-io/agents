use std::sync::Arc;

use iron::prelude::*;
use iron::status;
use iron::Handler;
use iron_json_response::JsonResponse;
use opentracingrust::Log;

use replicante_util_iron::request_span;
use replicante_util_tracing::fail_span;

use super::super::super::Agent;
use super::super::super::AgentContext;

/// Handler implementing the /api/v1/info/agent endpoint.
pub struct AgentInfo {
    agent: Arc<Agent>,
}

impl AgentInfo {
    pub fn make(agent: Arc<Agent>) -> AgentInfo {
        AgentInfo { agent }
    }
}

impl Handler for AgentInfo {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let mut span = request_span(request);
        span.log(Log::new().log("span.kind", "server-receive"));
        let info = self
            .agent
            .agent_info(&mut span)
            .map_err(|error| fail_span(error, &mut span))?;
        let mut response = Response::new();
        response
            .set_mut(JsonResponse::json(info))
            .set_mut(status::Ok);
        span.log(Log::new().log("span.kind", "server-send"));
        Ok(response)
    }
}

/// Handler implementing the /api/v1/info/datastore endpoint.
pub struct DatastoreInfo {
    agent: Arc<Agent>,
    context: AgentContext,
}

impl DatastoreInfo {
    pub fn make(agent: Arc<Agent>, context: AgentContext) -> DatastoreInfo {
        DatastoreInfo { agent, context }
    }
}

impl Handler for DatastoreInfo {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let mut span = request_span(request);
        span.log(Log::new().log("span.kind", "server-receive"));
        let mut info = self
            .agent
            .datastore_info(&mut span)
            .map_err(|error| fail_span(error, &mut span))?;

        // Inject the cluster_display_name override if configured.
        info.cluster_display_name = self
            .context
            .config
            .cluster_display_name_override
            .clone()
            .or(info.cluster_display_name);

        let mut response = Response::new();
        response
            .set_mut(JsonResponse::json(info))
            .set_mut(status::Ok);
        span.log(Log::new().log("span.kind", "server-send"));
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    mod agent {
        use std::sync::Arc;

        use iron::Chain;
        use iron::Headers;
        use iron::IronError;
        use iron_json_response::JsonResponseMiddleware;
        use iron_test::request;
        use iron_test::response;

        use replicante_util_iron::mock_request_span;

        use super::super::super::super::super::testing::MockAgent;
        use super::super::super::super::super::Agent;
        use super::super::super::super::super::AgentContext;
        use super::super::super::AgentInfo;

        fn get<A>(agent: A) -> Result<String, IronError>
        where
            A: Agent + 'static,
        {
            let handler = AgentInfo::make(Arc::new(agent));
            let handler = {
                let mut chain = Chain::new(handler);
                chain.link_after(JsonResponseMiddleware::new());
                chain
            };
            let handler = mock_request_span(AgentContext::mock().tracer, handler);
            let response = request::get(
                "http://localhost:3000/api/v1/info/agent",
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
        fn returns_error() {
            let mut agent = MockAgent::new();
            agent.agent_info = Err("Testing failure".into());
            let result = get(agent);
            assert!(result.is_err());
            if let Some(result) = result.err() {
                let body = response::extract_body_to_bytes(result.response);
                let body = String::from_utf8(body).unwrap();
                assert_eq!(
                    body,
                    r#"{"error":"Testing failure","layers":["Testing failure"],"trace":null}"#
                );
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

        use iron::Chain;
        use iron::Headers;
        use iron::IronError;
        use iron_json_response::JsonResponseMiddleware;
        use iron_test::request;
        use iron_test::response;

        use replicante_util_iron::mock_request_span;

        use super::super::super::super::super::config::Agent as AgentConfig;
        use super::super::super::super::super::testing::MockAgent;
        use super::super::super::super::super::Agent;
        use super::super::super::super::super::AgentContext;
        use super::super::super::DatastoreInfo;

        fn get<A>(agent: A) -> Result<String, IronError>
        where
            A: Agent + 'static,
        {
            get_with_config(agent, AgentConfig::default())
        }

        fn get_with_config<A>(agent: A, config: AgentConfig) -> Result<String, IronError>
        where
            A: Agent + 'static,
        {
            let context = AgentContext::mock_with_config(config);
            let handler = DatastoreInfo::make(Arc::new(agent), context);
            let handler = {
                let mut chain = Chain::new(handler);
                chain.link_after(JsonResponseMiddleware::new());
                chain
            };
            let handler = mock_request_span(AgentContext::mock().tracer, handler);
            let response = request::get(
                "http://localhost:3000/api/v1/info/datastore",
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
        fn override_display_name() {
            let agent = MockAgent::new();
            let config = AgentConfig::default();
            let result = get_with_config(agent, config).unwrap();
            let expected = concat!(
                r#"{"cluster_display_name":"display","cluster_id":"id","#,
                r#""kind":"DB","node_id":"mock","version":"1.2.3"}"#
            );
            assert_eq!(result, expected);
        }

        #[test]
        fn return_error() {
            let mut agent = MockAgent::new();
            agent.datastore_info = Err("Testing failure".into());
            let result = get(agent);
            assert!(result.is_err());
            if let Some(result) = result.err() {
                let body = response::extract_body_to_bytes(result.response);
                let body = String::from_utf8(body).unwrap();
                assert_eq!(
                    body,
                    r#"{"error":"Testing failure","layers":["Testing failure"],"trace":null}"#
                );
            }
        }

        #[test]
        fn return_version() {
            let agent = MockAgent::new();
            let result = get(agent).unwrap();
            let expected = concat!(
                r#"{"cluster_display_name":"display","cluster_id":"id","#,
                r#""kind":"DB","node_id":"mock","version":"1.2.3"}"#
            );
            assert_eq!(result, expected);
        }
    }
}
