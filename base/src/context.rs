use std::fmt;
use std::sync::Arc;

use opentracingrust::Tracer;
use prometheus::Registry;
#[cfg(any(test, feature = "with_test_support"))]
use slog::o;
#[cfg(any(test, feature = "with_test_support"))]
use slog::Discard;
use slog::Logger;

use crate::api::ApiAddons;
use crate::config::Agent as AgentConfig;
use crate::store::backend_factory;
use crate::store::Store;
use crate::Result;

/// Agent services injection.
///
/// A container to allow agents and the agent runner to access configured
/// sub-systems like logging, metrics, etc ...
// Cannot derive Debug because `Tracer` :-(
// Any new field must be added to the implementation of Debug.
#[derive(Clone)]
pub struct AgentContext {
    pub api_addons: ApiAddons<AgentContext>,
    pub config: AgentConfig,
    pub logger: Logger,

    /// Access the agent's metrics [`Registry`].
    ///
    /// Agents MUST register their metrics at creation time and as part of the same [`Registry`].
    ///
    /// [`Registry`]: https://docs.rs/prometheus/0.3.13/prometheus/struct.Registry.html
    pub metrics: Registry,

    /// Access the agent's persistent store.
    pub store: Store,

    /// Access the agent's [`Tracer`].
    ///
    /// This is the agent's way to access the opentracing compatible tracer.
    ///
    /// [`Tracer`]: https://docs.rs/opentracingrust/0.3.0/opentracingrust/struct.Tracer.html
    pub tracer: Arc<Tracer>,
}

impl fmt::Debug for AgentContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AgentContext")
            .field("config", &self.config)
            .field("logger", &self.logger)
            .field("metrics", &"<Registry>")
            .field("store", &"<Store>")
            .field("tracer", &"<Tracer>")
            .finish()
    }
}

impl AgentContext {
    pub fn new(config: AgentConfig, logger: Logger, tracer: Tracer) -> Result<AgentContext> {
        let metrics = Registry::new();
        let tracer = Arc::new(tracer);
        let store = backend_factory(&config, logger.clone(), Arc::clone(&tracer))?;
        Ok(AgentContext {
            api_addons: ApiAddons::default(),
            config,
            logger,
            metrics,
            store,
            tracer,
        })
    }

    #[cfg(any(test, feature = "with_test_support"))]
    pub fn mock() -> AgentContext {
        AgentContext::mock_with_config(AgentConfig::mock())
    }

    #[cfg(any(test, feature = "with_test_support"))]
    pub fn mock_with_config(config: AgentConfig) -> AgentContext {
        let mut upkeep = ::replicante_util_upkeep::Upkeep::new();
        let logger = Logger::root(Discard, o!());
        let metrics = Registry::new();
        let store = Store::mock();
        let opts = ::replicante_util_tracing::Opts::new("test", logger.clone(), &mut upkeep);
        let tracer =
            ::replicante_util_tracing::tracer(::replicante_util_tracing::Config::Noop, opts)
                .unwrap();
        let tracer = Arc::new(tracer);
        AgentContext {
            api_addons: ApiAddons::default(),
            config,
            logger,
            metrics,
            store,
            tracer,
        }
    }
}
