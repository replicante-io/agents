use std::collections::HashMap;

use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use chrono::DateTime;
use chrono::Utc;
use failure::Fail;
use failure::ResultExt;
use opentracingrust::ExtractFormat;
use opentracingrust::InjectFormat;
use opentracingrust::Span;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;
use serde::de::DeserializeOwned;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::json;
use serde_json::Value as Json;
use uuid::Uuid;

use crate::store::Transaction;
use crate::ErrorKind;
use crate::Result;

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

    /// Invoke the action to advance the given `ActionRecord`.
    fn invoke(
        &self,
        tx: &mut Transaction,
        record: &dyn ActionRecordView,
        span: Option<&mut Span>,
    ) -> Result<()>;

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

/// Summary info about an action returned in lists.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ActionListItem {
    pub action: String,
    pub id: Uuid,
    pub state: ActionState,
}

/// Action state and metadata information.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionRecord {
    /// Type ID of the action to run.
    pub action: String,

    /// Version of the agent that last validated the action.
    pub agent_version: String,

    /// Arguments passed to the action when invoked.
    pub args: Json,

    /// Time the agent recorded the action in the DB.
    pub created_ts: DateTime<Utc>,

    /// Additional metadata headers attached to the action.
    pub headers: HashMap<String, String>,

    /// Unique ID of the action.
    pub id: Uuid,

    /// Entity (system or user) requesting the execution of the action.
    pub requester: ActionRequester,

    /// State the action is currently in.
    state: ActionState,

    /// Optional payload attached to the current state.
    state_payload: Option<Json>,
}

impl ActionRecord {
    /// Construct an `ActionRecord` from raw attributes.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn inflate(
        action: String,
        agent_version: String,
        args: Json,
        created_ts: DateTime<Utc>,
        headers: HashMap<String, String>,
        id: Uuid,
        requester: ActionRequester,
        state: ActionState,
        state_payload: Option<Json>,
    ) -> ActionRecord {
        ActionRecord {
            action,
            agent_version,
            args,
            created_ts,
            headers,
            id,
            requester,
            state,
            state_payload,
        }
    }

    /// Initialise a new action to be executed.
    pub fn new<S>(action: S, args: Json, requester: ActionRequester) -> ActionRecord
    where
        S: Into<String>,
    {
        let action = action.into();
        ActionRecord {
            action,
            agent_version: env!("CARGO_PKG_VERSION").to_string(),
            args,
            created_ts: Utc::now(),
            headers: HashMap::new(),
            id: Uuid::new_v4(),
            requester,
            state: ActionState::New,
            state_payload: None,
        }
    }

    /// Extract the tracing context, if any is available.
    pub fn trace_get(&self, tracer: &Tracer) -> Result<Option<SpanContext>> {
        let format = ExtractFormat::TextMap(Box::new(&self.headers));
        let context = tracer
            .extract(format)
            .map_err(failure::SyncFailure::new)
            .with_context(|_| ErrorKind::ActionDecode)?;
        Ok(context)
    }

    /// Set the tracing context on the action record for propagation.
    pub fn trace_set(&mut self, context: &SpanContext, tracer: &Tracer) -> Result<()> {
        let format = InjectFormat::TextMap(Box::new(&mut self.headers));
        tracer
            .inject(context, format)
            .map_err(failure::SyncFailure::new)
            .with_context(|_| ErrorKind::ActionEncode)?;
        Ok(())
    }

    /// Test helper to set an action state.
    #[cfg(any(test, feature = "with_test_support"))]
    pub fn set_state(&mut self, state: ActionState) {
        self.state = state;
    }

    /// Test helper to set an action payload.
    #[cfg(any(test, feature = "with_test_support"))]
    pub fn set_state_payload(&mut self, payload: Option<Json>) {
        self.state_payload = payload;
    }
}

/// A dynamic view on `ActionRecord`s.
///
/// Allows actions to be composable by "presenting" the state a "nested action expects.
/// Look at the `replicante.service.restart` action for an example.
pub trait ActionRecordView {
    /// Access the raw record, mainly to pass it to the store interface.
    fn inner(&self) -> &ActionRecord;

    /// Manipulte the next state and payload to transition to.
    fn map_transition(
        &self,
        transition_to: ActionState,
        payload: Option<Json>,
    ) -> Result<(ActionState, Option<Json>)>;

    /// Access the state the action is currently in.
    fn state(&self) -> &ActionState;

    /// Access the associated state payload, if any.
    fn state_payload(&self) -> &Option<Json>;
}

impl ActionRecordView {
    /// Access the state as stored in the ActionRecord.
    pub fn raw_state(record: &ActionRecord) -> &ActionState {
        &record.state
    }

    /// Extract a structured payload, if any was stored for the action.
    pub fn structured_state_payload<T>(view: &dyn ActionRecordView) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let payload = view
            .state_payload()
            .clone()
            .map(serde_json::from_value)
            .transpose()
            .with_context(|_| ErrorKind::ActionDecode)?;
        Ok(payload)
    }
}

impl ActionRecordView for ActionRecord {
    fn inner(&self) -> &ActionRecord {
        self
    }

    fn map_transition(
        &self,
        transition_to: ActionState,
        payload: Option<Json>,
    ) -> Result<(ActionState, Option<Json>)> {
        Ok((transition_to, payload))
    }

    fn state(&self) -> &ActionState {
        &self.state
    }

    fn state_payload(&self) -> &Option<Json> {
        &self.state_payload
    }
}

/// Transition history records for actions.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionRecordHistory {
    /// ID of the action that transitioned.
    pub action_id: Uuid,

    /// Time the agent transitioned into this state.
    pub timestamp: DateTime<Utc>,

    /// State the action is currently in.
    pub state: ActionState,

    /// Optional payload attached to the current state.
    pub state_payload: Option<Json>,
}

/// Entity (system, user, ...) that requested the action to be performed.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ActionRequester {
    Api,
}

/// Current state of an action execution.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ActionState {
    /// The action is to be cancelled, but that has not happened yet.
    Cancel,

    /// The action was successfully cancelled.
    Cancelled,

    /// The action was successfully completed.
    Done,

    /// The action ended with an error.
    Failed,

    /// The action has just been sheduled and is not being executed yet.
    New,

    /// The action was started by the agent and is in progress.
    Running,
}

impl ActionState {
    /// True if the action is finished (failed or succeeded).
    pub fn is_finished(&self) -> bool {
        match self {
            ActionState::Cancelled => true,
            ActionState::Done => true,
            ActionState::Failed => true,
            _ => false,
        }
    }
}

/// Result alias for methods that return an ActionValidityError.
pub type ActionValidity<T = ()> = std::result::Result<T, ActionValidityError>;

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
