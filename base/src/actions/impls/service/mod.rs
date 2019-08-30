use serde::Deserialize;
use serde::Serialize;

use crate::actions::ACTIONS;
use crate::Agent;
use crate::AgentContext;

mod composed;
mod start;
mod stop;
mod supervisor;

use self::composed::ServiceRestart;
use self::start::ServiceStart;
use self::stop::ServiceStop;

/// Persisted progress of service start/stop actions.
#[derive(Serialize, Deserialize)]
struct ServiceActionState {
    attempt: u8,
    message: Option<String>,
    pid: Option<String>,
}

impl Default for ServiceActionState {
    fn default() -> Self {
        ServiceActionState {
            attempt: 0,
            message: None,
            pid: None,
        }
    }
}

/// Register all service related actions.
pub fn register(agent: &dyn Agent, context: &AgentContext) {
    let supervisor = self::supervisor::factory(agent, context);
    ACTIONS::register_reserved(ServiceStart::new(&supervisor));
    ACTIONS::register_reserved(ServiceStop::new(&supervisor));
    ACTIONS::register_reserved(ServiceRestart::make(&supervisor));
}
