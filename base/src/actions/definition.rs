use std::collections::HashMap;

use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use chrono::DateTime;
use chrono::Utc;
use failure::Fail;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::json;
use serde_json::Value as Json;
use uuid::Uuid;

/// Abstraction of any action the agent can perform.
///
/// # Action Kinds
/// Action Kinds must be scoped to limit the chance of clashes.
/// Scoping is done using the `<SCOPE>.<ACTION>` format.
/// An action kind can have as many `.`s in it as desired, making Java-like reverse DNS
/// scopes an option that greatly reduces the chances of clashes.
///
/// The only constraint on Action Kindss is some scopes are reserved to replicante use itself.
/// This allows the base agent frameworks to define some standard actions across all agents
/// without clashing with custom or database specific actions.
pub trait Action: Send + Sync + 'static {
    /// Action metadata and attributes.
    fn describe(&self) -> ActionDescriptor;

    /// Validate the arguments passed to an action request.
    fn validate_args(&self, args: &Json) -> ActionValidity;
}

/// Container for an action's metadata and other attributes.
///
/// This data is the base of the actions system.
/// Instead of hardcoded knowledge about what actions do,
/// both system and users rely on metadata to call actions.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ActionDescriptor {
    pub kind: String,
    pub description: String,
}

/// Action state and metadata information.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionRecord {
    pub action: String,
    pub agent_version: String,
    pub args: Json,
    pub created_ts: DateTime<Utc>,
    pub headers: HashMap<String, String>,
    pub id: Uuid,
    pub requester: ActionRequester,
    pub state: ActionState,
}

impl ActionRecord {
    pub fn new(action: String, args: Json, requester: ActionRequester) -> ActionRecord {
        ActionRecord {
            action,
            agent_version: env!("CARGO_PKG_VERSION").to_string(),
            args,
            created_ts: Utc::now(),
            headers: HashMap::new(),
            id: Uuid::new_v4(),
            requester,
            state: ActionState::New,
        }
    }
}

/// Entity (system, user, ...) that requested the action to be performed.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ActionRequester {
    #[serde(rename = "API")]
    Api,
}

/// Current state of an action execution.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ActionState {
    #[serde(rename = "NEW")]
    New,
}

/// Result alias for methods that return an ActionValidityError.
pub type ActionValidity<T = ()> = Result<T, ActionValidityError>;

/// Result of action validation process.
#[derive(Debug, Fail)]
pub enum ActionValidityError {
    #[fail(display = "invalid action arguments: {}", _0)]
    InvalidArgs(String),
}

impl ActionValidityError {
    fn kind(&self) -> &str {
        match self {
            ActionValidityError::InvalidArgs(_) => "InvalidArgs",
        }
    }
}

impl ActionValidityError {
    fn http_status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl ResponseError for ActionValidityError {
    fn error_response(&self) -> HttpResponse {
        let status = self.http_status();
        HttpResponse::build(status).json(json!({
            "error": self.to_string(),
            "kind": self.kind(),
        }))
    }

    fn render_response(&self) -> HttpResponse {
        self.error_response()
    }
}
