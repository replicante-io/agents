use opentracingrust::Span;
use serde_json::Value as Json;

use crate::actions::Action;
use crate::actions::ActionDescriptor;
use crate::actions::ActionRecordView;
use crate::actions::ActionState;
use crate::actions::ActionValidity;
use crate::store::Transaction;
use crate::Result;

/// Do nothing but transition to `ActionState::Done`, optionally with a payload.
pub struct NoOp {
    payload: Option<Json>,
}

impl NoOp {
    /// New action, with optional payload.
    pub fn new<J>(payload: J) -> NoOp
    where
        J: Into<Option<Json>>,
    {
        let payload = payload.into();
        NoOp { payload }
    }
}

impl Action for NoOp {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "replicante.noop".into(),
            description: "Do nothing but transition to done".into(),
        }
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        tx.action().transition(
            record,
            ActionState::Done,
            self.payload.clone(),
            span.map(|span| span.context().clone()),
        )
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::NoOp;
    use crate::actions::Action;
    use crate::actions::ActionRecord;
    use crate::actions::ActionRecordView;
    use crate::actions::ActionRequester;
    use crate::actions::ActionState;
    use crate::store::Store;

    /// Helper function to create a new clean record.
    fn mkrecord(action: &dyn Action) -> ActionRecord {
        let kind = action.describe().kind;
        ActionRecord::new(kind, None, None, json!(null), ActionRequester::Api)
    }

    #[test]
    fn invoke_action() {
        let action = NoOp::new(json!({
            "attr": 123,
            "message": "test",
        }));
        let store = Store::mock();
        store
            .with_transaction(|tx| {
                let record = mkrecord(&action);
                let record_id = record.id.to_string();
                tx.action().insert(record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                action.invoke(tx, &record, None)?;
                let record = tx.action().get(&record_id, None)?.unwrap();
                assert_eq!(*record.state(), ActionState::Done);
                let payload = record.state_payload().clone().unwrap();
                assert_eq!(
                    payload,
                    json!({
                        "attr": 123,
                        "message": "test",
                    })
                );
                Ok(())
            })
            .unwrap();
    }
}
