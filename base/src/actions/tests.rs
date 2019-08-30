use actix_web::dev::Body;
use actix_web::dev::Service;
use actix_web::test;
use actix_web::test::TestRequest;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use failure::Fail;
use opentracingrust::Span;
use serde_json::json;
use serde_json::Value as Json;

use super::Action;
use super::ActionDescriptor;
use super::ActionRecordView;
use super::ActionState;
use super::ActionValidity;
use super::ActionValidityError;
use crate::config::Agent as Config;
use crate::config::TlsConfig;
use crate::store::Transaction;
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
