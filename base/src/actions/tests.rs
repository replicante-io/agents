use actix_web::dev::Body;
use actix_web::dev::Service;
use actix_web::test;
use actix_web::test::TestRequest;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use failure::Fail;
use failure::ResultExt;
use opentracingrust::Span;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value as Json;

use super::Action;
use super::ActionDescriptor;
use super::ActionRecord;
use super::ActionRecordView;
use super::ActionRequester;
use super::ActionState;
use super::ActionValidity;
use super::ActionValidityError;
use crate::config::Agent as Config;
use crate::config::TlsConfig;
use crate::store::Store;
use crate::store::Transaction;
use crate::ErrorKind;
use crate::Result;

struct TestAction {}

impl Action for TestAction {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "replicante.test.action1".into(),
            description: "Replicante test action 1".into(),
        }
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        _: Option<&mut Span>,
    ) -> Result<()> {
        tx.action()
            .transition(record, ActionState::Done, Json::from(42), None)
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Err(ActionValidityError::InvalidArgs("test".into()))
    }
}

struct TestActionFail {}

impl Action for TestActionFail {
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "replicante.test.action2".into(),
            description: "Replicante test action 2".into(),
        }
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        _: Option<&mut Span>,
    ) -> Result<()> {
        tx.action()
            .transition(record, ActionState::Failed, Json::from(21), None)
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}

struct TestComposedAction<A1, A2>
where
    A1: Action,
    A2: Action,
{
    first: A1,
    second: A2,
}

impl<A1, A2> Action for TestComposedAction<A1, A2>
where
    A1: Action,
    A2: Action,
{
    fn describe(&self) -> ActionDescriptor {
        ActionDescriptor {
            kind: "replicante.test.composed".into(),
            description: "Test action composing two other actions".into(),
        }
    }

    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()> {
        let state: TestComposedActionPayload =
            ActionRecordView::structured_state_payload(record)?.unwrap_or_default();
        let view = TestComposedActionView {
            record,
            state: &state,
        };
        if state.stage == 0 {
            return self.first.invoke(tx, &view, span);
        }
        self.second.invoke(tx, &view, span)
    }

    fn validate_args(&self, _: &Json) -> ActionValidity {
        Ok(())
    }
}

struct TestComposedActionView<'a> {
    record: &'a dyn ActionRecordView,
    state: &'a TestComposedActionPayload,
}

impl<'a> ActionRecordView for TestComposedActionView<'a> {
    fn inner(&self) -> &ActionRecord {
        self.record.inner()
    }

    fn map_transition(
        &self,
        transition_to: ActionState,
        payload: Option<Json>,
    ) -> Result<(ActionState, Option<Json>)> {
        let (mut transition_to, payload) = self.record.map_transition(transition_to, payload)?;
        let mut composed_payload = self.state.clone();
        if self.state.stage == 0 {
            composed_payload.first = payload;
            composed_payload.first_state = transition_to.clone();
        } else {
            composed_payload.second = payload;
            composed_payload.second_state = transition_to.clone();
        }
        if transition_to == ActionState::Done && composed_payload.stage == 0 {
            composed_payload.stage = 1;
            transition_to = ActionState::Running;
        }
        let payload =
            serde_json::to_value(composed_payload).with_context(|_| ErrorKind::ActionEncode)?;
        Ok((transition_to, Some(payload)))
    }

    fn state(&self) -> &ActionState {
        if self.state.stage == 0 {
            &self.state.first_state
        } else {
            &self.state.second_state
        }
    }

    fn state_payload(&self) -> &Option<Json> {
        if self.state.stage == 0 {
            &self.state.first
        } else {
            &self.state.second
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct TestComposedActionPayload {
    first: Option<Json>,
    first_state: ActionState,
    second: Option<Json>,
    second_state: ActionState,
    stage: u8,
}

impl Default for TestComposedActionPayload {
    fn default() -> Self {
        TestComposedActionPayload {
            first: None,
            first_state: ActionState::New,
            second: None,
            second_state: ActionState::New,
            stage: 0,
        }
    }
}

#[test]
fn composed_action_failed() {
    let store = Store::mock();
    store
        .with_transaction(|tx| {
            let action = TestComposedAction {
                first: TestActionFail {},
                second: TestAction {},
            };
            let record =
                ActionRecord::new(action.describe().kind, json!(null), ActionRequester::Api);
            let record_id = record.id.to_string();
            tx.action().insert(record, None)?;
            let record = tx.action().get(&record_id, None)?.unwrap();
            action.invoke(tx, &record, None)?;
            let record = tx.action().get(&record_id, None)?.unwrap();
            assert_eq!(*record.state(), ActionState::Failed);
            assert_eq!(
                *record.state_payload(),
                Some(json!({
                    "first": 21,
                    "first_state": "FAILED",
                    "second": null,
                    "second_state": "NEW",
                    "stage": 0,
                }))
            );
            Ok(())
        })
        .unwrap();
}

#[test]
fn composed_action_success() {
    let store = Store::mock();
    store
        .with_transaction(|tx| {
            let action = TestComposedAction {
                first: TestAction {},
                second: TestAction {},
            };
            let record =
                ActionRecord::new(action.describe().kind, json!(null), ActionRequester::Api);
            let record_id = record.id.to_string();
            tx.action().insert(record, None)?;
            let record = tx.action().get(&record_id, None)?.unwrap();
            action.invoke(tx, &record, None)?;
            let record = tx.action().get(&record_id, None)?.unwrap();
            assert_eq!(*record.state(), ActionState::Running);
            assert_eq!(
                *record.state_payload(),
                Some(json!({
                    "first": 42,
                    "first_state": "DONE",
                    "second": null,
                    "second_state": "NEW",
                    "stage": 1,
                }))
            );
            action.invoke(tx, &record, None)?;
            let record = tx.action().get(&record_id, None)?.unwrap();
            assert_eq!(*record.state(), ActionState::Done);
            assert_eq!(
                *record.state_payload(),
                Some(json!({
                    "first": 42,
                    "first_state": "DONE",
                    "second": 42,
                    "second_state": "DONE",
                    "stage": 1,
                }))
            );
            Ok(())
        })
        .unwrap();
}

#[test]
fn disabled_by_default() {
    let config = Config::mock();
    let enabled = super::actions_enabled(&config);
    assert!(!enabled.unwrap(), "actions should be disabled by default");
}

#[test]
fn disabled_explicitly_with_tls() {
    let mut config = Config::mock();
    let tls = TlsConfig {
        clients_ca_bundle: Some("clients".to_string()),
        server_cert: "server.crt".to_string(),
        server_key: "server.key".to_string(),
    };
    config.actions.enabled = Some(false);
    config.api.tls = Some(tls);
    let enabled = super::actions_enabled(&config);
    assert!(!enabled.unwrap(), "actions should be disabled by config");
}

#[test]
fn enabled_implicitly_by_tls() {
    let mut config = Config::mock();
    let tls = TlsConfig {
        clients_ca_bundle: Some("clients".to_string()),
        server_cert: "server.crt".to_string(),
        server_key: "server.key".to_string(),
    };
    config.api.tls = Some(tls);
    let enabled = super::actions_enabled(&config);
    assert!(
        enabled.unwrap(),
        "actions should be enabled by clients bundle",
    );
}

#[test]
fn enabled_explicitly_without_tls() {
    let mut config = Config::mock();
    config.actions.enabled = Some(true);
    match super::actions_enabled(&config) {
        Ok(_) => panic!("expected configuration error"),
        Err(error) => assert_eq!(error.name().unwrap(), "ConfigClash"),
    };
}

#[test]
fn validation_fails() {
    let mut app = test::init_service(App::new().route(
        "/",
        web::get().to(|| -> actix_web::Result<HttpResponse> {
            let action = TestAction {};
            action.validate_args(&json!({}))?;
            Ok(HttpResponse::Ok().json(json!({})))
        }),
    ));
    let req = TestRequest::get().uri("/").to_request();
    let mut resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status().as_u16(), 400);
    let body = match resp.take_body().as_ref().unwrap() {
        Body::Bytes(body) => String::from_utf8(body.to_vec()).unwrap(),
        _ => panic!("invalid body type"),
    };
    assert_eq!(
        body,
        r#"{"error":"invalid action arguments: test","kind":"InvalidArgs"}"#
    );
}
