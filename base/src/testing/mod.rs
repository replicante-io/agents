use opentracingrust::Span;
use opentracingrust::SpanReceiver;
use opentracingrust::Tracer;
use opentracingrust::tracers::NoopTracer;

use prometheus::Registry;

use super::Agent;
use super::AgentResult;

use replicante_agent_models::AgentVersion;
use replicante_agent_models::DatastoreInfo;
use replicante_agent_models::Shard;


/// An implementation of Agent to be used for tests.
pub struct MockAgent {
    // Mock responses
    pub datastore_info: AgentResult<DatastoreInfo>,
    pub shards: AgentResult<Vec<Shard>>,

    // Introspection
    registry: Registry,
    tracer: Tracer,
}

impl MockAgent {
    /// Creates a new [`NoopTracer`] and a MockAgent that uses it.
    ///
    /// The method returns both the mock agent and a [`SpanReceiver`].
    /// The caller will need to keep the [`SpanReceiver`] alive for the duration of the test.
    ///
    /// [`NoopTracer`]: tracers/struct.NoopTracer.html
    /// [`SpanReceiver`]: type.SpanReceiver.html
    pub fn new() -> (MockAgent, SpanReceiver) {
        let (tracer, receiver) = NoopTracer::new();
        let agent = MockAgent::new_with_tracer(tracer);
        (agent, receiver)
    }

    /// Creates a new MockAgent that uses the given tracer.
    pub fn new_with_tracer(tracer: Tracer) -> MockAgent {
        MockAgent {
            // Mock responses
            datastore_info: Ok(DatastoreInfo::new("DB", "mock", "1.2.3")),
            shards: Ok(vec![]),

            // Introspection
            registry: Registry::new(),
            tracer: tracer
        }
    }
}

impl Agent for MockAgent {
    fn agent_version(&self, _: &mut Span) -> AgentResult<AgentVersion> {
        Ok(AgentVersion::new("dcd", "1.2.3", "tainted"))
    }

    fn datastore_info(&self, _: &mut Span) -> AgentResult<DatastoreInfo> {
        self.datastore_info.clone()
    }

    fn shards(&self, _:&mut Span) -> AgentResult<Vec<Shard>> {
        self.shards.clone()
    }

    fn metrics(&self) -> Registry {
        self.registry.clone()
    }

    fn tracer(&self) -> &Tracer {
        &self.tracer
    }
}
