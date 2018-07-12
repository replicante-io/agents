use std::sync::Arc;
use std::sync::RwLock;

use opentracingrust::Span;

use replicante_agent_models::AgentInfo;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shards;

use super::Agent;
use super::Result;


/// Information about an Agent that is active.
#[derive(Clone)]
pub struct ActiveAgent {
    agent: Arc<Agent>,
    remake_on_error: bool,
    version_id: String,
}

impl ActiveAgent {
    /// Instantiate a new ActiveAgent.
    ///
    /// The active agent stores the following information:
    ///
    ///   * The `Agent` implementation to forward method calls too.
    ///   * `remake_on_error` to indicate that `AgentFactory::make` should be called on error.
    ///   * The `version_id` opaque string to be used by `AgentFactory::should_remake`
    ///     to determine the ID of the active agent version.
    pub fn new<S: Into<String>>(
        agent: Arc<Agent>, remake_on_error: bool, version_id: S
    ) -> ActiveAgent {
        ActiveAgent {
            agent,
            remake_on_error,
            version_id: version_id.into(),
        }
    }

    pub fn remake_on_error(&self) -> bool {
        self.remake_on_error
    }

    pub fn version_id(&self) -> &String {
        &self.version_id
    }
}


/// Abstract logic to instantiate an Agent.
pub trait AgentFactory : Send + Sync {
    /// Instantiate a new Agent best interacting with the current version.
    ///
    /// This method should detect the best fit communicating with the datastore itself.
    fn make(&self) -> ActiveAgent;

    /// Checks if the currently active agent should be replaced with a new one.
    fn should_remake(&self, active: &ActiveAgent, info: &DatastoreInfo) -> bool;
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
/// The last feature (remake_on_error) allows implementing an "unsupported version" agent
/// that can fail fast and be replaced as soon as possible (especially useful if the
/// datastore is down and a default agent cannot be defined).
///
/// *Be aware that* the request will only be performed by the active agent.
/// The new agent, if any, will become active for future requests only.
///
///
/// # Default versions
/// When building versioned agents there are two cases you must account for:
///
///   * Starting the agent when the datastore is not running.
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
/// The default behaviour is to replace the agent implementation, if needed,
/// only as part of the `Agent::datastore_info` method.
///
/// This may lead to different agent implementations being used to handle
/// different agent methods and can introduce delays in agent replacement.
///
/// Agents using the `VersionedAgent` facility may install a background
/// thread that periodically invokes the `Agent::datastore_info` method.
///
/// Such thread would cause the agent implementation to be kept up to date
/// even if no calls to `Agent::datastore_info` are made.
pub struct VersionedAgent {
    active: RwLock<ActiveAgent>,
    factory: Box<AgentFactory>,
}

impl VersionedAgent {
    pub fn new<F: AgentFactory + 'static>(factory: F) -> VersionedAgent {
        let factory = Box::new(factory);
        let active = RwLock::new(factory.make());
        VersionedAgent {
            active,
            factory,
        }
    }

    /// Replace the active agent with a newly made one.
    pub fn remake_agent(&self) {
        let new_active = self.factory.make();
        let mut active = self.active.write().expect("ActiveAgent lock was poisoned");
        *active = new_active;
    }
}

impl Agent for VersionedAgent {
    fn agent_info(&self, span: &mut Span) -> Result<AgentInfo> {
        let active = self.active.read().expect("ActiveAgent lock was poisoned");
        active.agent.agent_info(span)
    }

    fn datastore_info(&self, span: &mut Span) -> Result<DatastoreInfo> {
        let active = self.active.read().expect("ActiveAgent lock was poisoned").clone();
        let info = active.agent.datastore_info(span);
        match info {
            Err(error) => {
                if active.remake_on_error {
                    self.remake_agent();
                }
                Err(error)
            },
            Ok(info) => {
                if self.factory.should_remake(&active, &info) {
                    self.remake_agent();
                }
                Ok(info)
            }
        }
    }

    fn shards(&self, span: &mut Span) -> Result<Shards> {
        let active = self.active.read().expect("ActiveAgent lock was poisoned");
        active.agent.shards(span)
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;

    use opentracingrust::Span;

    use replicante_agent_models::AgentInfo;
    use replicante_agent_models::DatastoreInfo;
    use replicante_agent_models::Shards;

    use super::super::AgentContext;
    use super::super::Result;
    use super::super::testing::MockAgent;

    use super::ActiveAgent;
    use super::Agent;
    use super::AgentFactory;
    use super::VersionedAgent;


    struct MockFactory {
        pub agent: Arc<Agent>,
        pub made: Mutex<i32>,
        pub remake: bool,
        pub remake_on_error: bool,
    }
    impl AgentFactory for MockFactory {
        fn make(&self) -> ActiveAgent {
            let mut made = self.made.lock().unwrap();
            *made += 1;
            drop(made);
            ActiveAgent::new(Arc::clone(&self.agent), self.remake_on_error, "test")
        }

        fn should_remake(&self, _: &ActiveAgent, _: &DatastoreInfo) -> bool {
            self.remake
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

        fn shards(&self, span:&mut Span) -> Result<Shards> {
            self.0.shards(span)
        }
    }

    struct WrappedMockFactory(Arc<MockFactory>);
    impl AgentFactory for WrappedMockFactory {
        fn make(&self) -> ActiveAgent { self.0.make() }
        fn should_remake(&self, active: &ActiveAgent, info: &DatastoreInfo) -> bool {
            self.0.should_remake(active, info)
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
        let agent = VersionedAgent::new(WrappedMockFactory(Arc::clone(&factory)));
        assert_eq!(1, *factory.made.lock().unwrap());
        let (context, extra) = AgentContext::mock();
        agent.datastore_info(&mut context.tracer.span("TEST")).unwrap();
        assert_eq!(1, *factory.made.lock().unwrap());
        drop(extra);
    }

    #[test]
    fn remake() {
        let factory = Arc::new(MockFactory {
            agent: Arc::new(MockAgent::new()),
            made: Mutex::new(0),
            remake: true,
            remake_on_error: false,
        });
        let agent = VersionedAgent::new(WrappedMockFactory(Arc::clone(&factory)));
        assert_eq!(1, *factory.made.lock().unwrap());
        let (context, extra) = AgentContext::mock();
        agent.datastore_info(&mut context.tracer.span("TEST")).unwrap();
        assert_eq!(2, *factory.made.lock().unwrap());
        drop(extra);
    }

    #[test]
    fn remake_on_error() {
        let mut mocked = MockAgent::new();
        mocked.datastore_info = Err("test".into());
        let mocked = Arc::new(mocked);
        let factory = Arc::new(MockFactory {
            agent: Arc::new(WrappedMockAgent(Arc::clone(&mocked))),
            made: Mutex::new(0),
            remake: false,
            remake_on_error: true,
        });
        let agent = VersionedAgent::new(WrappedMockFactory(Arc::clone(&factory)));
        assert_eq!(1, *factory.made.lock().unwrap());
        let (context, extra) = AgentContext::mock();
        let info = agent.datastore_info(&mut context.tracer.span("TEST"));
        assert!(info.is_err());
        assert_eq!(2, *factory.made.lock().unwrap());
        drop(extra);
    }
}
