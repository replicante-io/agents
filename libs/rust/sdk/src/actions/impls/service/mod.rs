use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;

use crate::actions::Action;
use crate::actions::ACTIONS;
use crate::AgentContext;

mod composed;
mod start;
mod stop;
mod supervisor;

use self::composed::GracefulRestart;
use self::composed::GracefulStop;
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
pub fn register(context: &AgentContext, graceful: Option<Arc<dyn Action>>) {
    let service = match &context.config.service {
        None => return,
        Some(service) => service.clone(),
    };
    let supervisor = self::supervisor::factory(&context.logger, service);
    ACTIONS::register_reserved(GracefulRestart::make(graceful.clone(), &supervisor));
    ACTIONS::register_reserved(GracefulStop::make(graceful, &supervisor));
    ACTIONS::register_reserved(ServiceRestart::make(&supervisor));
    ACTIONS::register_reserved(ServiceStart::new(&supervisor));
    ACTIONS::register_reserved(ServiceStop::new(&supervisor));
}
