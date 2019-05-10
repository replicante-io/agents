use std::fmt;
use std::sync::Arc;

#[cfg(debug_assertions)]
use std::time::Duration;

use opentracingrust::Tracer;
use prometheus::Registry;

#[cfg(debug_assertions)]
use slog::Discard;
use slog::Logger;

#[cfg(debug_assertions)]
use replicante_util_tracing::TracerExtra;

use super::config::Agent as AgentConfig;

/// Agent services injection.
///
/// A container to allow agents and the agent runner to access configured
/// sub-systems like logging, metrics, etc ...
// Cannot derive Debug because `Tracer` :-(
// Any new field must be added to the implementation of Debug.
#[derive(Clone)]
pub struct AgentContext {
    pub config: AgentConfig,
    pub logger: Logger,

    /// Acess the agent's metrics [`Registry`].
    ///
    /// Agents MUST register their metrics at creation time and as part of the same [`Registry`].
    ///
    /// [`Registry`]: https://docs.rs/prometheus/0.3.13/prometheus/struct.Registry.html
    pub metrics: Registry,

    /// Access the agent's [`Tracer`].
    ///
    /// This is the agent's way to access the opentracing compatible tracer.
    ///
    /// [`Tracer`]: https://docs.rs/opentracingrust/0.3.0/opentracingrust/struct.Tracer.html
    pub tracer: Arc<Tracer>,
}

impl fmt::Debug for AgentContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AgentContext{{config:{:?},logger:{:?},metrics:Registry,tracer:Tracer}}",
            self.config, self.logger,
        )
    }
}

impl AgentContext {
    pub fn new(config: AgentConfig, logger: Logger, tracer: Tracer) -> AgentContext {
        let metrics = Registry::new();
        AgentContext {
            config,
            logger,
            metrics,
            tracer: Arc::new(tracer),
        }
    }

    #[cfg(debug_assertions)]
    pub fn mock() -> (AgentContext, TracerExtra) {
        AgentContext::mock_with_config(AgentConfig::default())
    }

    #[cfg(debug_assertions)]
    pub fn mock_with_config(config: AgentConfig) -> (AgentContext, TracerExtra) {
        let logger = Logger::root(Discard, o!());
        let (tracer, mut extra) = ::replicante_util_tracing::tracer(
            ::replicante_util_tracing::Config::Noop,
            logger.clone(),
        )
        .unwrap();
        let context = AgentContext::new(config, logger, tracer);
        if let TracerExtra::ReporterThread(ref mut reporter) = extra {
            reporter.stop_delay(Duration::from_millis(2));
        };
        (context, extra)
    }
}
