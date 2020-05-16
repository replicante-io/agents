use bson::doc;
use mongodb::sync::Client;
use opentracingrust::Span;
use serde_json::json;
use serde_json::Value as Json;

use replicante_agent::actions::Action;
use replicante_agent::actions::ActionDescriptor;
use replicante_agent::actions::ActionHook;
use replicante_agent::actions::ActionRecordView;
use replicante_agent::actions::ActionState;
use replicante_agent::actions::ActionValidity;
use replicante_agent::Result;
use replicante_agent::Transaction;

/// Request graceful server stop by issuing a `shutdown` command.
pub struct GracefulStop {
    client: Client,
}

impl GracefulStop {
    pub fn new(client: Client) -> GracefulStop {
        GracefulStop { client }
    }
}

impl Action for GracefulStop {
    fn describe(&self) -> ActionDescriptor {
        ActionHook::StoreGracefulStop.describe()
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        let shutdown = doc! {"shutdown": 1};
        // This will fail even on success as the server will not respond.
        let result = self.client.database("admin").run_command(shutdown, None);
        let message = format!("{:?}", result);
        let payload = json!({ "message": message });
        tx.action().transition(
            record,
            ActionState::Done,
            payload,
            span.map(|span| span.context().clone()),
        )
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}
