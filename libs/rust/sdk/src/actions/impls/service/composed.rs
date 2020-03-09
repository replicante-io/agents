use std::sync::Arc;

use serde_json::json;

use super::supervisor::Supervisor;
use super::ServiceStart;
use super::ServiceStop;
use crate::actions::advanced::AndThen;
use crate::actions::advanced::NoOp;
use crate::actions::ActionDescriptor;
use crate::Agent;

const GRACEFUL_NOT_SUPPORTED: &str = "graceful stop not supported by the datastore";
const GRACEFULRESTART_DESCRIPTION: &str =
    "Gracefully stop the datastore, if supported, and stop/start the service";
const GRACEFULSTOP_DESCRIPTION: &str =
    "Gracefully stop the datastore, if supported, and stop the service";

/// Gracefully stop the datastore, if supported, and stop/start the service.
pub struct GracefulRestart {}

impl GracefulRestart {
    pub fn make(agent: &dyn Agent, supervisor: &Arc<dyn Supervisor>) -> AndThen {
        let graceful = match agent.graceful_stop_action() {
            None => Arc::new(NoOp::new(json!(GRACEFUL_NOT_SUPPORTED))),
            Some(action) => action,
        };
        AndThen::build()
            .describe(ActionDescriptor {
                kind: "replicante.io/service.gracefulrestart".into(),
                description: GRACEFULRESTART_DESCRIPTION.into(),
            })
            .and_then_arc(graceful, "graceful")
            .and_then(ServiceStop::new(supervisor), "stop")
            .and_then(ServiceStart::new(supervisor), "start")
            .finish()
    }
}

/// Gracefully stop the datastore, if supported, and stop the service.
pub struct GracefulStop {}

impl GracefulStop {
    pub fn make(agent: &dyn Agent, supervisor: &Arc<dyn Supervisor>) -> AndThen {
        let graceful = match agent.graceful_stop_action() {
            None => Arc::new(NoOp::new(json!(GRACEFUL_NOT_SUPPORTED))),
            Some(action) => action,
        };
        AndThen::build()
            .describe(ActionDescriptor {
                kind: "replicante.io/service.gracefulstop".into(),
                description: GRACEFULSTOP_DESCRIPTION.into(),
            })
            .and_then_arc(graceful, "graceful")
            .and_then(ServiceStop::new(supervisor), "stop")
            .finish()
    }
}

/// Composed action to `ServiceStop` & `ServiceStart`.
pub struct ServiceRestart {}

impl ServiceRestart {
    pub fn make(supervisor: &Arc<dyn Supervisor>) -> AndThen {
        AndThen::build()
            .describe(ActionDescriptor {
                kind: "replicante.io/service.restart".into(),
                description: "Stop/Start the datstore service".into(),
            })
            .and_then(ServiceStop::new(supervisor), "stop")
            .and_then(ServiceStart::new(supervisor), "start")
            .finish()
    }
}
