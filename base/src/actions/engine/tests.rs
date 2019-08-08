use serde_json::json;

use replicante_util_failure::SerializableFail;

use super::super::debug::Progress;
use super::Engine;
use crate::actions::ActionRecord;
use crate::actions::ActionRequester;
use crate::actions::ActionState;
use crate::actions::ActionsRegister;
use crate::actions::ACTIONS;
use crate::AgentContext;

#[test]
fn fail_action_with_unkown_kind() {
    let action = ActionRecord::new("test".to_string(), json!({}), ActionRequester::Api);
    let id = action.id;
    let context = AgentContext::mock();
    context
        .store
        .with_transaction(|tx| tx.action().insert(action, None))
        .unwrap();
    let register = ActionsRegister::default();
    ACTIONS::test_with(register, || {
        let engine = Engine::new(context.clone());
        engine.poll().expect("poll failed to process action");
    });
    let action = context
        .store
        .with_transaction(|tx| tx.action().get(&id.to_string(), None))
        .unwrap()
        .unwrap();
    assert_eq!(id, action.id);
    assert_eq!(ActionState::Failed, action.state);
    let payload = action.state_payload.expect("need a state payload");
    let payload: SerializableFail = serde_json::from_value(payload).unwrap();
    assert_eq!(payload.error, "actions with kind test are not available");
}

#[test]
fn no_action_noop() {
    let context = AgentContext::mock();
    let engine = Engine::new(context);
    engine.poll().expect("poll failed to process action");
}

#[test]
fn transition_new_to_running() {
    let action = ActionRecord::new(
        "replicante.debug.progress".to_string(),
        json!({}),
        ActionRequester::Api,
    );
    let id = action.id;
    let context = AgentContext::mock();
    context
        .store
        .with_transaction(|tx| tx.action().insert(action, None))
        .unwrap();
    let mut register = ActionsRegister::default();
    register.register_reserved(Progress {});
    ACTIONS::test_with(register, || {
        let engine = Engine::new(context.clone());
        engine.poll().expect("poll failed to process action");
    });
    let action = context
        .store
        .with_transaction(|tx| tx.action().get(&id.to_string(), None))
        .unwrap()
        .unwrap();
    assert_eq!(id, action.id);
    assert_eq!(ActionState::Running, action.state);
}

// TODO: action transitions from running to success.
// TODO: running actions are picked before new ones.
