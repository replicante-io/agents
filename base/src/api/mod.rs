use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use failure::ResultExt;
use humthreads::Builder;
use iron::Chain;
use iron::Iron;
use iron_json_response::JsonResponseMiddleware;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;
use replicante_util_iron::MetricsMiddleware;
use replicante_util_iron::RequestLogger;
use replicante_util_iron::RootDescriptor;
use replicante_util_iron::Router;
use replicante_util_iron::SentryMiddleware;
use replicante_util_upkeep::Upkeep;

mod agent;
mod index;
mod introspect;
mod metrics;

pub use self::metrics::register_metrics;

use super::config::SentryCaptureApi;
use super::Agent;
use super::AgentContext;
use super::ErrorKind;
use super::Result;

/// Mount all API endpoints into an Iron Chain.
pub fn mount(agent: Arc<Agent>, context: AgentContext) -> Chain {
    let logger = context.logger.clone();
    let mut router = Router::new(context.config.api.trees.clone().into());
    let sentry_capture_api = context
        .config
        .sentry
        .as_ref()
        .map(|sentry| sentry.capture_api_errors.clone())
        .unwrap_or_default();

    // Create the index root for each API root.
    let roots = [APIRoot::UnstableAPI];
    for root in roots.iter() {
        let mut root = router.for_root(root);
        root.get("/", index::index, "index");
    }

    // Mount endpooints.
    self::introspect::mount(&context, &mut router);
    self::agent::mount(agent, context, &mut router);

    // Build and return the Iron Chain.
    let mut chain = router.build();
    let metrics_middlewere = MetricsMiddleware::new(
        self::metrics::MIDDLEWARE.0.clone(),
        self::metrics::MIDDLEWARE.1.clone(),
        self::metrics::MIDDLEWARE.2.clone(),
        logger.clone(),
    );
    chain.link_after(JsonResponseMiddleware::new());
    chain.link_after(RequestLogger::new(logger));
    match sentry_capture_api {
        SentryCaptureApi::Client => {
            chain.link_after(SentryMiddleware::new(400));
        }
        SentryCaptureApi::No => (),
        SentryCaptureApi::Server => {
            chain.link_after(SentryMiddleware::new(500));
        }
    };
    chain.link(metrics_middlewere.into_middleware());
    chain
}

/// Start an Iron HTTP server.
///
/// # Panics
///
/// This method panics if:
///
///   * It fails to bind to the configured port.
pub fn spawn_server<A>(agent: A, context: AgentContext, upkeep: &mut Upkeep) -> Result<()>
where
    A: Agent + 'static,
{
    let agent: Arc<dyn Agent> = Arc::new(agent);
    let thread = Builder::new("r:b:api")
        .full_name("replicante:base:api")
        .spawn(move |scope| {
            let chain = mount(Arc::clone(&agent), context.clone());
            let config = &context.config.api;
            let mut server = Iron::new(chain);
            server.timeouts.keep_alive = config.timeouts.keep_alive.map(Duration::from_secs);
            server.timeouts.read = config.timeouts.read.map(Duration::from_secs);
            server.timeouts.write = config.timeouts.write.map(Duration::from_secs);
            if let Some(threads_count) = config.threads_count {
                server.threads = threads_count;
            }

            info!(context.logger, "Starting API server"; "bind" => &config.bind);
            scope.activity("running https://github.com/iron/iron HTTP server");
            let mut bind = server
                .http(&config.bind)
                .expect("Unable to start API server");
            // Once started, the server will run in the background.
            // When the guard returned by Iron::http is dropped it tries to join the server.
            // To support shutting down wait for the signal here, then close the server.
            // NOTE: closing the server does not really work, just prevent the need to join :-(
            //   See https://github.com/hyperium/hyper/issues/338
            while !scope.should_shutdown() {
                ::std::thread::sleep(Duration::from_secs(1));
            }
            if let Err(error) = bind.close() {
                capture_fail!(
                    &error,
                    context.logger,
                    "Failed to shutdown API server";
                    failure_info(&error),
                );
            }
        })
        .with_context(|_| ErrorKind::ThreadSpawn("api server"))?;
    upkeep.register_thread(thread);
    Ok(())
}

/// Enumerates all possible API roots.
///
/// All endpoints must fall under one of these roots and are subject to all restrictions
/// of that specific root.
/// The main restriction is that versioned APIs are subject to semver guarantees.
pub enum APIRoot {
    /// API root for all endpoints that are not yet stable.
    ///
    /// Endpoints in this root are NOT subject to ANY compatibility guarantees!
    UnstableAPI,

    /// Instrospection APIs not yet stable.
    UnstableIntrospect,
}

impl RootDescriptor for APIRoot {
    fn enabled(&self, flags: &HashMap<&'static str, bool>) -> bool {
        match self {
            APIRoot::UnstableAPI => match flags.get("unstable") {
                Some(flag) => *flag,
                None => true,
            },
            APIRoot::UnstableIntrospect => match flags.get("unstable") {
                Some(flag) if !flag => false,
                _ => match flags.get("introspect") {
                    Some(flag) => *flag,
                    None => true,
                },
            },
        }
    }

    fn prefix(&self) -> &'static str {
        match self {
            APIRoot::UnstableAPI => "/api/unstable",
            APIRoot::UnstableIntrospect => "/api/unstable/introspect",
        }
    }
}
