use std::sync::Arc;
use std::sync::RwLock;

use opentracingrust::Log;
use opentracingrust::Span;
use slog::debug;
use slog::info;
use slog::warn;

use replicante_models_agent::info::AgentInfo;
use replicante_models_agent::info::DatastoreInfo;
use replicante_models_agent::info::Shards;
use replicante_util_failure::failure_info;

use crate::actions::Action;
use crate::actions::ActionHook;
use crate::Agent;
use crate::AgentContext;
use crate::Error;
use crate::Result;

/// Information about an Agent that is active.
#[derive(Clone)]
pub struct ActiveAgent {
    agent: Arc<dyn Agent>,
    version_id: String,
}

impl ActiveAgent {
    /// Instantiate a new ActiveAgent.
    ///
    /// The active agent stores the following information:
    ///
    ///   * The `Agent` implementation to forward method calls too.
    ///   * The `version_id` opaque string to be used by `AgentFactory::should_remake`
    ///     to determine the ID of the active agent version.
    pub fn new<S: Into<String>>(agent: Arc<dyn Agent>, version_id: S) -> ActiveAgent {
        ActiveAgent {
            agent,
            version_id: version_id.into(),
        }
    }

    pub fn version_id(&self) -> &String {
        &self.version_id
    }
}

/// Abstract logic to instantiate an Agent.
pub trait AgentFactory: Send + Sync {
    /// Instantiate a new Agent best interacting with the current version.
    ///
    /// This method should detect the best fit communicating with the datastore itself.
    fn make(&self) -> ActiveAgent;

    /// Checks if the currently active agent should be replaced with a new one.
    fn should_remake(&self, active: &ActiveAgent, info: &DatastoreInfo) -> bool;

    /// Checks if the currently active agent should be replaced with a new one in case of error.
    fn should_remake_on_error(&self, active: &ActiveAgent, error: &Error) -> bool;
}

/// Replicante agent decorator to support runtime-selected agent versions.
///
/// This agent implements logic to switch the behaviour based on the
/// current version of the datastore.
///
/// Useful for agents that want to support multiple datastore versions and change as they do:
///
///   * Agents are created by and AgentFactory.
///   * Each agent has a version_id assigned.
///   * Every call to the `Agent::datastore_info` method is a chance to replace the agent.
///   * The active agent is replaced if the factory says so.
///   * The factory can also indicate that the agent should be remade on error.
///
/// The last feature (should_remake_on_error) allows implementing an "unsupported version"
/// agent that can fail fast and be replaced as soon as possible (especially useful if the
/// datastore is down and a default agent cannot be defined).
///
/// If the agent is replaced by the version check a new request for data is issued
/// even if that may result in an unnecessary call to the database.
///
///
/// # Default versions
/// When building versioned agents there are two cases you must account for:
///
///   * Starting the agent when the datastore is not running or broken.
///   * Running the agent against an unsupported version of the datastore.
///
/// The easiest way to deal with these cases is to introduce a default version: an agent that
/// implements the bare minimum it can that is compatible with all supported versions.
///
/// When the `AgentFactory` has to instantiate an agent but can't reliably determine
/// what the version of the datastore is it can use this default version.
///
///
/// # Forcing a version change
/// To help with consistency, the version is checked every call and only calls to
/// `VersionedAgent::datastore_info`.
///
/// Agents applications can implement additional strategies by calling
/// `VersionedAgent::validate_version`.
pub struct VersionedAgent<Factory>
where
    Factory: AgentFactory + 'static,
{
    active: RwLock<ActiveAgent>,
    context: AgentContext,
    factory: Factory,
}

impl<Factory> VersionedAgent<Factory>
where
    Factory: AgentFactory + 'static,
{
    /// Replace the active agent with a newly made one.
    fn remake_agent(&self, span: &mut Span) {
        span.log(Log::new().log("message", "VersionedAgent remakes the agent"));
        span.tag("agent.remade", true);
        let new_active = self.factory.make();
        let mut active = self.active.write().expect("ActiveAgent lock was poisoned");
        *active = new_active;
    }
}

impl<Factory> VersionedAgent<Factory>
where
    Factory: AgentFactory + 'static,
{
    pub fn new(context: AgentContext, factory: Factory) -> VersionedAgent<Factory> {
        let active = RwLock::new(factory.make());
        VersionedAgent {
            active,
            context,
            factory,
        }
    }

    /// Check if the active agent should be replaced.
    ///
    /// Grabs version information from the current database and checks if a more appropriate
    /// agent implementation can be instantiated.
    /// If so, one is instantiated and activated immediatelly.
    ///
    /// If the active agent if determined to be the most appropriate for the current
    /// datastore version the fetched `DatastoreInfo` object is returned
    /// and nothing else is changed.
    pub fn validate_version(&self, span: &mut Span) -> Option<DatastoreInfo> {
        // Scope version check because it requires a read lock.
        let (should_remake, info) = {
            let active = self
                .active
                .read()
                .expect("ActiveAgent lock was poisoned")
                .clone();
            let info = active.agent.datastore_info(span);
            match info {
                Err(error) => {
                    warn!(self.context.logger, "Failed to detect version"; failure_info(&error));
                    (self.factory.should_remake_on_error(&active, &error), None)
                }
                Ok(info) => (self.factory.should_remake(&active, &info), Some(info)),
            }
        };
        // Remake the agent if needed.
        if should_remake {
            debug!(self.context.logger, "Remaking versioned agent");
            self.remake_agent(span);
            info!(self.context.logger, "Versioned agent re-made");
            return None;
        }
        info
    }
}

impl<Factory> Agent for VersionedAgent<Factory>
where
    Factory: AgentFactory + 'static,
{
    fn agent_info(&self, span: &mut Span) -> Result<AgentInfo> {
        let active = self.active.read().expect("ActiveAgent lock was poisoned");
        active.agent.agent_info(span)
    }

    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
        // If validation returns a version we can reuse that in the response.
        if let Some(info) = self.validate_version(span) {
            return Ok(info);
        }
        // Otherwise we attempt to get it directly.
        let active = self
            .active
            .read()
            .expect("ActiveAgent lock was poisoned")
            .clone();
        active.agent.datastore_info(span)
    }

    fn shards(&self, span: &mut Span) -> Result<Shards> {
        let active = self.active.read().expect("ActiveAgent lock was poisoned");
        active.agent.shards(span)
    }

    fn action_hooks(&self) -> Vec<(ActionHook, Arc<dyn Action>)> {
        let active = self.active.read().expect("ActiveAgent lock was poisoned");
        active.agent.action_hooks()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;

    use opentracingrust::Span;

    use replicante_models_agent::info::AgentInfo;
    use replicante_models_agent::info::DatastoreInfo;
    use replicante_models_agent::info::Shards;

    use super::super::testing::MockAgent;
    use super::super::AgentContext;
    use super::super::Error;
    use super::super::Result;

    use super::ActiveAgent;
    use super::Agent;
    use super::AgentFactory;
    use super::VersionedAgent;

    struct MockFactory {
        pub agent: Arc<dyn Agent>,
        pub made: Mutex<i32>,
        pub remake: bool,
        pub remake_on_error: bool,
    }
    impl AgentFactory for MockFactory {
        fn make(&self) -> ActiveAgent {
            let mut made = self.made.lock().unwrap();
            *made += 1;
            drop(made);
            ActiveAgent::new(Arc::clone(&self.agent), "test")
        }

        fn should_remake(&self, _: &ActiveAgent, _: &DatastoreInfo) -> bool {
            self.remake
        }

        fn should_remake_on_error(&self, _: &ActiveAgent, _: &Error) -> bool {
            self.remake_on_error
        }
    }

    struct WrappedMockAgent(Arc<MockAgent>);
    impl Agent for WrappedMockAgent {
        fn agent_info(&self, span: &mut Span) -> Result<AgentInfo> {
            self.0.agent_info(span)
        }

        fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
            self.0.datastore_info(span)
        }

        fn shards(&self, span: &mut Span) -> Result<Shards> {
            self.0.shards(span)
        }
    }

    struct WrappedMockFactory(Arc<MockFactory>);
    impl AgentFactory for WrappedMockFactory {
        fn make(&self) -> ActiveAgent {
            self.0.make()
        }

        fn should_remake(&self, active: &ActiveAgent, info: &DatastoreInfo) -> bool {
            self.0.should_remake(active, info)
        }
        fn should_remake_on_error(&self, active: &ActiveAgent, error: &Error) -> bool {
            self.0.should_remake_on_error(active, error)
        }
    }

    #[test]
    fn does_not_remake() {
        let factory = Arc::new(MockFactory {
            agent: Arc::new(MockAgent::new()),
            made: Mutex::new(0),
            remake: false,
            remake_on_error: false,
        });
        let context = AgentContext::mock();
        let agent = VersionedAgent::new(context.clone(), WrappedMockFactory(Arc::clone(&factory)));
        assert_eq!(1, *factory.made.lock().unwrap());
        agent
            .datastore_info(&mut context.tracer.span("TEST"))
            .unwrap();
        assert_eq!(1, *factory.made.lock().unwrap());
    }

    #[test]
    fn validate_version_info_error() {
        let mut mocked = MockAgent::new();
        mocked.datastore_info = Err("test".into());
        let mocked = Arc::new(mocked);
        let factory = Arc::new(MockFactory {
            agent: Arc::new(WrappedMockAgent(Arc::clone(&mocked))),
            made: Mutex::new(0),
            remake: false,
            remake_on_error: true,
        });
        let context = AgentContext::mock();
        let agent = VersionedAgent::new(context.clone(), WrappedMockFactory(Arc::clone(&factory)));
        agent.validate_version(&mut context.tracer.span("TEST"));
        assert_eq!(2, *factory.made.lock().unwrap());
    }

    #[test]
    fn validate_version_info_error_no_change() {
        let mut mocked = MockAgent::new();
        mocked.datastore_info = Err("test".into());
        let mocked = Arc::new(mocked);
        let factory = Arc::new(MockFactory {
            agent: Arc::new(WrappedMockAgent(Arc::clone(&mocked))),
            made: Mutex::new(0),
            remake: false,
            remake_on_error: false,
        });
        let context = AgentContext::mock();
        let agent = VersionedAgent::new(context.clone(), WrappedMockFactory(Arc::clone(&factory)));
        agent.validate_version(&mut context.tracer.span("TEST"));
        assert_eq!(1, *factory.made.lock().unwrap());
    }

    #[test]
    fn validate_version_no_change() {
        let mocked = MockAgent::new();
        let mocked = Arc::new(mocked);
        let factory = Arc::new(MockFactory {
            agent: Arc::new(WrappedMockAgent(Arc::clone(&mocked))),
            made: Mutex::new(0),
            remake: false,
            remake_on_error: false,
        });
        let context = AgentContext::mock();
        let agent = VersionedAgent::new(context.clone(), WrappedMockFactory(Arc::clone(&factory)));
        agent.validate_version(&mut context.tracer.span("TEST"));
        assert_eq!(1, *factory.made.lock().unwrap());
    }

    #[test]
    fn validate_version_should_remake() {
        let mocked = MockAgent::new();
        let mocked = Arc::new(mocked);
        let factory = Arc::new(MockFactory {
            agent: Arc::new(WrappedMockAgent(Arc::clone(&mocked))),
            made: Mutex::new(0),
            remake: true,
            remake_on_error: false,
        });
        let context = AgentContext::mock();
        let agent = VersionedAgent::new(context.clone(), WrappedMockFactory(Arc::clone(&factory)));
        agent.validate_version(&mut context.tracer.span("TEST"));
        assert_eq!(2, *factory.made.lock().unwrap());
    }
}
