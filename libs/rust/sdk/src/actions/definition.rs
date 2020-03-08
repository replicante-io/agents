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

use replicante_models_agent::actions::ActionModel;

// Use the view versions of these models from here so they can easily change if needed.
pub use replicante_models_agent::actions::ActionHistoryItem;
pub use replicante_models_agent::actions::ActionListItem;
pub use replicante_models_agent::actions::ActionRequester;
pub use replicante_models_agent::actions::ActionState;

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
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct ActionDescriptor {
    pub kind: String,
    pub description: String,
}

/// Action state and metadata information.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionRecord {
    /// Version of the agent that last validated the action.
    pub agent_version: String,

    /// Time the action was first created (by the agent, by core, ...).
    pub created_ts: DateTime<Utc>,

    /// Time the action entered a finished state.
    pub finished_ts: Option<DateTime<Utc>>,

    /// Additional metadata headers attached to the action.
    pub headers: HashMap<String, String>,

    /// Unique ID of the action.
    pub id: Uuid,

    /// Type ID of the action to run.
    pub kind: String,

    /// Entity (system or user) requesting the execution of the action.
    pub requester: ActionRequester,

    /// Time the agent recorded the action in the DB.
    pub scheduled_ts: DateTime<Utc>,

    /// Arguments passed to the action when invoked.
    args: Json,

    /// State the action is currently in.
    state: ActionState,

    /// Optional payload attached to the current state.
    state_payload: Option<Json>,
}

impl ActionRecord {
    /// Construct an `ActionRecord` from raw attributes.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn inflate(
        agent_version: String,
        args: Json,
        created_ts: DateTime<Utc>,
        finished_ts: Option<DateTime<Utc>>,
        headers: HashMap<String, String>,
        id: Uuid,
        kind: String,
        requester: ActionRequester,
        scheduled_ts: DateTime<Utc>,
        state: ActionState,
        state_payload: Option<Json>,
    ) -> ActionRecord {
        ActionRecord {
            agent_version,
            args,
            created_ts,
            finished_ts,
            headers,
            id,
            kind,
            requester,
            scheduled_ts,
            state,
            state_payload,
        }
    }

    /// Initialise a new action to be executed.
    pub fn new<S>(
        kind: S,
        id: Option<Uuid>,
        created_ts: Option<DateTime<Utc>>,
        args: Json,
        requester: ActionRequester,
    ) -> ActionRecord
    where
        S: Into<String>,
    {
        let kind = kind.into();
        let id = id.unwrap_or_else(Uuid::new_v4);
        let created_ts = created_ts.unwrap_or_else(Utc::now);
        ActionRecord {
            agent_version: env!("CARGO_PKG_VERSION").to_string(),
            args,
            created_ts,
            finished_ts: None,
            headers: HashMap::new(),
            id,
            kind,
            requester,
            scheduled_ts: Utc::now(),
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

impl From<ActionRecord> for ActionModel {
    fn from(record: ActionRecord) -> ActionModel {
        ActionModel {
            args: record.args,
            created_ts: record.created_ts,
            finished_ts: record.finished_ts,
            headers: record.headers,
            id: record.id,
            kind: record.kind,
            requester: record.requester,
            scheduled_ts: record.scheduled_ts,
            state: record.state,
            state_payload: record.state_payload,
        }
    }
}

/// A dynamic view on `ActionRecord`s.
///
/// Allows actions to be composable by "presenting" the state a "nested action expects.
/// Look at the `replicante.service.restart` action for an example.
pub trait ActionRecordView {
    /// Access the action arguments.
    fn args(&self) -> &Json;

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

impl dyn ActionRecordView {
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
    fn args(&self) -> &Json {
        &self.args
    }

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
