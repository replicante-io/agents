use std::convert::TryInto;

use chrono::Utc;
use opentracingrust::Span;
use serde_json::Value as Json;

use crate::actions::Action;
use crate::actions::ActionDescriptor;
use crate::actions::ActionRecordView;
use crate::actions::ActionState;
use crate::actions::ActionValidity;
use crate::store::Transaction;
use crate::ErrorKind;
use crate::Result;

/// Test action that emits pong messages.
pub struct Ping {}

impl Action for Ping {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "agent.replicante.io/test.ping".into(),
            description: "Test action that emits pong messages".into(),
        }
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        let mut payload = match record.state_payload() {
            None => Vec::new(),
            Some(Json::Array(payload)) => payload.clone(),
            Some(state) => {
                let error = format!("invalid payload: expect array, found {}", state);
                return Err(ErrorKind::FreeForm(error).into());
            }
        };

        let now = Utc::now();
        let message = format!("Pong at {}", now);
        payload.push(message.into());

        let count = match record.args() {
            Json::Null => None,
            Json::Object(args) => args.get("count"),
            args => {
                let error = format!("invalid arguments: expect object, found {}", args);
                return Err(ErrorKind::FreeForm(error).into());
            }
        };
        let count = match count {
            None => None,
            Some(Json::Number(count)) => Some(count),
            Some(count) => {
                let error = format!("invalid count: expect usize, found {}", count);
                return Err(ErrorKind::FreeForm(error).into());
            }
        };
        let count = match count.map(|count| count.as_u64()) {
            None => 1,
            Some(Some(count)) => count,
            Some(None) => {
                let error = format!("invalid count: expect usize, found {}", count.unwrap());
                return Err(ErrorKind::FreeForm(error).into());
            }
        };
        let count = match count.try_into() {
            Ok(count) => count,
            Err(error) => {
                let error = format!("invalid count: expect usize, err: {}", error);
                return Err(ErrorKind::FreeForm(error).into());
            }
        };
        let next_state = if payload.len() < count {
            ActionState::Running
        } else {
            ActionState::Done
        };
        tx.action().transition(
            record,
            next_state,
            Json::from(payload),
            span.map(|span| span.context().clone()),
        )
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}
