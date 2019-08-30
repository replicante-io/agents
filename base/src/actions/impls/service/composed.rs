use std::sync::Arc;

use crate::actions::advanced::AndThen;
use crate::actions::ActionDescriptor;

use super::supervisor::Supervisor;
use super::ServiceStart;
use super::ServiceStop;

pub struct ServiceRestart {}

impl ServiceRestart {
    pub fn make(supervisor: &Arc<dyn Supervisor>) -> AndThen {
        AndThen::build()
            .describe(ActionDescriptor {
                kind: "replicante.service.restart".into(),
                description: "Stop/Start the datstore service".into(),
            })
            .and_then(ServiceStop::new(supervisor), "stop")
            .and_then(ServiceStart::new(supervisor), "start")
            .finish()
    }
}
