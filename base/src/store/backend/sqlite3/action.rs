use std::str::FromStr;

use chrono::TimeZone;
use chrono::Utc;
use failure::ResultExt;
use opentracingrust::SpanContext;
use opentracingrust::StartOptions;
use rusqlite::params;
use rusqlite::Row;
use rusqlite::NO_PARAMS;
use serde_json::Value as Json;
use uuid::Uuid;

use replicante_util_tracing::MaybeTracer;

use crate::actions::ActionRecord;
use crate::actions::ActionRecordHistory;
use crate::actions::ActionState;
use crate::metrics::SQLITE_OPS_COUNT;
use crate::metrics::SQLITE_OPS_DURATION;
use crate::metrics::SQLITE_OP_ERRORS_COUNT;
use crate::store::interface::ActionInterface;
use crate::store::Iter;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

const ACTION_GET: &str = "action.get";
const ACTION_GET_SQL: &str = r#"
SELECT
    action,
    agent_version,
    args,
    created_ts,
    headers,
    id,
    requester,
    state,
    state_payload
FROM actions
WHERE id = ?;
"#;
const ACTION_GET_HISTORY: &str = "action.get.history";
const ACTION_GET_HISTORY_SQL: &str = r#"
SELECT
    action_id,
    time,
    state,
    state_payload
FROM actions_history
WHERE action_id = ?;
"#;
const ACTION_INSERT: &str = "action.insert";
const ACTION_INSERT_SQL: &str = r#"
INSERT INTO actions (
    action,
    agent_version,
    args,
    created_ts,
    headers,
    id,
    requester,
    state,
    state_payload
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);
"#;
const ACTION_INSERT_HISTORY: &str = "action.insert.history";
const ACTION_INSERT_HISTORY_SQL: &str = r#"
INSERT INTO actions_history (
    action_id,
    time,
    state,
    state_payload
)
VALUES (?1, ?2, ?3, ?4);
"#;
const ACTION_NEXT: &str = "action.next";
const ACTION_NEXT_SQL: &str = r#"
SELECT
    action,
    agent_version,
    args,
    created_ts,
    headers,
    id,
    requester,
    state,
    state_payload
FROM actions
WHERE finished_ts IS NULL
ORDER BY created_ts
LIMIT 1;
"#;
const ACTION_TRANSITION: &str = "action.transition";
const ACTION_TRANSITION_SQL: &str = r#"
UPDATE actions
SET
    state = ?1,
    state_payload = ?2,
    finished_ts = ?3
WHERE id = ?4;
"#;

/// Helper macro to avoid writing the same match every time.
macro_rules! decode_or_continue {
    ($decode:expr, $res:ident, $op:expr $(,)?) => {
        match $decode {
            Ok(r) => r,
            Err(error) => {
                let error = Err(error)
                    .with_context(|_| ErrorKind::PersistentRead($op))
                    .map_err(Error::from);
                $res.push(error);
                continue;
            }
        }
    };
}

/// Helper macro to avoid writing the same match every time.
macro_rules! decode_or_return {
    ($decode:expr, $op:expr $(,)?) => {
        match $decode {
            Ok(r) => r,
            Err(error) => {
                let error = Err(error)
                    .with_context(|_| ErrorKind::PersistentRead($op))
                    .map_err(Error::from);
                return error;
            }
        }
    };
}

/// Parse a SQLite result row into a full ActionRecord.
fn parse_action(row: &Row, op: &'static str) -> Result<ActionRecord> {
    let id: String = decode_or_return!(row.get("id"), op);
    let id = decode_or_return!(Uuid::from_str(&id), op);
    let action: String = decode_or_return!(row.get("action"), op);
    let agent_version: String = decode_or_return!(row.get("agent_version"), op);
    let args: String = decode_or_return!(row.get("args"), op);
    let args = decode_or_return!(serde_json::from_str(&args), op);
    let created_ts: i64 = decode_or_return!(row.get("created_ts"), op);
    let created_ts = Utc.timestamp(created_ts, 0);
    let headers: String = decode_or_return!(row.get("headers"), op);
    let headers = decode_or_return!(serde_json::from_str(&headers), op);
    let requester: String = decode_or_return!(row.get("requester"), op);
    let requester = decode_or_return!(serde_json::from_str(&requester), op);
    let state: String = decode_or_return!(row.get("state"), op);
    let state = decode_or_return!(serde_json::from_str(&state), op);
    let state_payload: Option<String> = decode_or_return!(row.get("state_payload"), op);
    let state_payload = match state_payload {
        None => None,
        Some(payload) => decode_or_return!(serde_json::from_str(&payload), op),
    };
    Ok(ActionRecord {
        action,
        agent_version,
        args,
        created_ts,
        headers,
        id,
        requester,
        state,
        state_payload,
    })
}

pub struct Action<'a, 'b: 'a> {
    inner: &'a rusqlite::Transaction<'b>,
    tracer: MaybeTracer,
}

impl<'a, 'b: 'a> Action<'a, 'b> {
    pub fn new(inner: &'a rusqlite::Transaction<'b>, tracer: MaybeTracer) -> Action<'a, 'b> {
        Action { inner, tracer }
    }

    fn record_transition(
        &self,
        action_id: String,
        state: String,
        state_payload: Option<String>,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let _span = self.tracer.with(|tracer| {
            let mut opts = StartOptions::default();
            if let Some(context) = span {
                opts = opts.child_of(context);
            }
            let mut span = tracer.span_with_options("store.sqlite.insert", opts);
            span.tag("sql", ACTION_INSERT_SQL);
            span.auto_finish()
        });
        SQLITE_OPS_COUNT.with_label_values(&["INSERT"]).inc();
        let _timer = SQLITE_OPS_DURATION
            .with_label_values(&["INSERT"])
            .start_timer();
        let mut statement = self
            .inner
            .prepare_cached(ACTION_INSERT_HISTORY_SQL)
            .with_context(|_| ErrorKind::PersistentWrite(ACTION_INSERT_HISTORY))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["INSERT"]).inc();
                error
            })?;
        statement
            .execute(params![
                action_id,
                Utc::now().timestamp(),
                state,
                state_payload,
            ])
            .with_context(|_| ErrorKind::PersistentWrite(ACTION_INSERT_HISTORY))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["INSERT"]).inc();
                error
            })?;
        Ok(())
    }
}

impl<'a, 'b: 'a> ActionInterface for Action<'a, 'b> {
    fn get(&self, id: &str, span: Option<SpanContext>) -> Result<Option<ActionRecord>> {
        let _span = self.tracer.with(|tracer| {
            let mut opts = StartOptions::default();
            if let Some(context) = span {
                opts = opts.child_of(context);
            }
            let mut span = tracer.span_with_options("store.sqlite.select", opts);
            span.tag("sql", ACTION_GET_SQL);
            span.auto_finish()
        });
        SQLITE_OPS_COUNT.with_label_values(&["SELECT"]).inc();
        let timer = SQLITE_OPS_DURATION
            .with_label_values(&["SELECT"])
            .start_timer();
        let mut statement = self
            .inner
            .prepare_cached(ACTION_GET_SQL)
            .with_context(|_| ErrorKind::PersistentRead(ACTION_GET))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["SELECT"]).inc();
                error
            })?;
        let mut rows = statement
            .query(params![id])
            .with_context(|_| ErrorKind::PersistentRead(ACTION_GET))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["SELECT"]).inc();
                error
            })?;
        let row = rows
            .next()
            .with_context(|_| ErrorKind::PersistentRead(ACTION_GET))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["SELECT"]).inc();
                error
            })?;
        timer.observe_duration();
        let row = match row {
            None => return Ok(None),
            Some(row) => row,
        };
        parse_action(row, ACTION_GET).map(Some)
    }

    fn history(&self, id: &str, span: Option<SpanContext>) -> Result<Iter<ActionRecordHistory>> {
        let _span = self.tracer.with(|tracer| {
            let mut opts = StartOptions::default();
            if let Some(context) = span {
                opts = opts.child_of(context);
            }
            let mut span = tracer.span_with_options("store.sqlite.select", opts);
            span.tag("sql", ACTION_GET_SQL);
            span.auto_finish()
        });
        SQLITE_OPS_COUNT.with_label_values(&["SELECT"]).inc();
        let _timer = SQLITE_OPS_DURATION
            .with_label_values(&["SELECT"])
            .start_timer();
        let mut statement = self
            .inner
            .prepare_cached(ACTION_GET_HISTORY_SQL)
            .with_context(|_| ErrorKind::PersistentRead(ACTION_GET_HISTORY))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["SELECT"]).inc();
                error
            })?;
        let mut results = Vec::new();
        let mut rows = statement
            .query(params![id])
            .with_context(|_| ErrorKind::PersistentRead(ACTION_GET_HISTORY))?;
        let mut maybe_row = rows
            .next()
            .with_context(|_| ErrorKind::PersistentRead(ACTION_GET_HISTORY))?;
        while let Some(row) = maybe_row {
            let action_id: String =
                decode_or_continue!(row.get("action_id"), results, ACTION_GET_HISTORY);
            let action_id =
                decode_or_continue!(Uuid::from_str(&action_id), results, ACTION_GET_HISTORY);
            let timestamp: i64 = decode_or_continue!(row.get("time"), results, ACTION_GET_HISTORY);
            let timestamp = Utc.timestamp(timestamp, 0);
            let state: String = decode_or_continue!(row.get("state"), results, ACTION_GET_HISTORY);
            let state: ActionState =
                decode_or_continue!(serde_json::from_str(&state), results, ACTION_GET_HISTORY);
            let state_payload: Option<String> =
                decode_or_continue!(row.get("state_payload"), results, ACTION_GET_HISTORY);
            let state_payload = match state_payload {
                None => None,
                Some(payload) => {
                    decode_or_continue!(serde_json::from_str(&payload), results, ACTION_GET_HISTORY)
                }
            };
            results.push(Ok(ActionRecordHistory {
                action_id,
                timestamp,
                state,
                state_payload,
            }));
            maybe_row = rows
                .next()
                .with_context(|_| ErrorKind::PersistentRead(ACTION_GET_HISTORY))?;
        }
        Ok(Iter::new(results.into_iter()))
    }

    fn insert(&self, action: ActionRecord, span: Option<SpanContext>) -> Result<()> {
        let span = self.tracer.with(|tracer| {
            let mut opts = StartOptions::default();
            if let Some(context) = span {
                opts = opts.child_of(context);
            }
            let mut span = tracer.span_with_options("store.sqlite.insert", opts);
            span.tag("sql", ACTION_INSERT_SQL);
            span.auto_finish()
        });
        let action_id = action.id.to_string();
        let args = serde_json::to_string(&action.args)
            .with_context(|_| ErrorKind::PersistentWrite(ACTION_INSERT))?;
        let headers = serde_json::to_string(&action.headers)
            .with_context(|_| ErrorKind::PersistentWrite(ACTION_INSERT))?;
        let requester = serde_json::to_string(&action.requester)
            .with_context(|_| ErrorKind::PersistentWrite(ACTION_INSERT))?;
        let state = serde_json::to_string(&action.state)
            .with_context(|_| ErrorKind::PersistentWrite(ACTION_INSERT))?;
        let state_payload = action
            .state_payload
            .map(|payload| {
                serde_json::to_string(&payload)
                    .with_context(|_| ErrorKind::PersistentWrite(ACTION_INSERT))
                    .map_err(Error::from)
            })
            .transpose()?;
        SQLITE_OPS_COUNT.with_label_values(&["INSERT"]).inc();
        let _timer = SQLITE_OPS_DURATION
            .with_label_values(&["INSERT"])
            .start_timer();
        let mut statement = self
            .inner
            .prepare_cached(ACTION_INSERT_SQL)
            .with_context(|_| ErrorKind::PersistentWrite(ACTION_INSERT))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["INSERT"]).inc();
                error
            })?;
        statement
            .execute(params![
                action.action,
                action.agent_version,
                args,
                action.created_ts.timestamp(),
                headers,
                &action_id,
                requester,
                &state,
                &state_payload,
            ])
            .with_context(|_| ErrorKind::PersistentWrite(ACTION_INSERT))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["INSERT"]).inc();
                error
            })?;
        self.record_transition(
            action_id,
            state,
            state_payload,
            span.map(|span| span.context().clone()),
        )?;
        Ok(())
    }

    fn next(&self, span: Option<SpanContext>) -> Result<Option<ActionRecord>> {
        let _span = self.tracer.with(|tracer| {
            let mut opts = StartOptions::default();
            if let Some(context) = span {
                opts = opts.child_of(context);
            }
            let mut span = tracer.span_with_options("store.sqlite.select", opts);
            span.tag("sql", ACTION_NEXT_SQL);
            span.auto_finish()
        });
        SQLITE_OPS_COUNT.with_label_values(&["SELECT"]).inc();
        let timer = SQLITE_OPS_DURATION
            .with_label_values(&["SELECT"])
            .start_timer();
        let mut statement = self
            .inner
            .prepare_cached(ACTION_NEXT_SQL)
            .with_context(|_| ErrorKind::PersistentRead(ACTION_NEXT))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["SELECT"]).inc();
                error
            })?;
        let mut rows = statement
            .query(NO_PARAMS)
            .with_context(|_| ErrorKind::PersistentRead(ACTION_NEXT))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["SELECT"]).inc();
                error
            })?;
        let row = rows
            .next()
            .with_context(|_| ErrorKind::PersistentRead(ACTION_NEXT))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["SELECT"]).inc();
                error
            })?;
        timer.observe_duration();
        let row = match row {
            None => return Ok(None),
            Some(row) => row,
        };
        parse_action(row, ACTION_NEXT).map(Some)
    }

    fn transition(
        &self,
        action: &ActionRecord,
        transition_to: ActionState,
        payload: Option<Json>,
        span: Option<SpanContext>,
    ) -> Result<()> {
        let span = self.tracer.with(|tracer| {
            let mut opts = StartOptions::default();
            if let Some(context) = span {
                opts = opts.child_of(context);
            }
            let mut span = tracer.span_with_options("store.sqlite.update", opts);
            span.tag("sql", ACTION_TRANSITION_SQL);
            span.auto_finish()
        });
        let finished_ts = if transition_to.is_finished() {
            Some(Utc::now().timestamp())
        } else {
            None
        };
        let action_id = action.id.to_string();
        let state = serde_json::to_string(&transition_to)
            .with_context(|_| ErrorKind::PersistentWrite(ACTION_TRANSITION))?;
        let state_payload = payload
            .map(|payload| {
                serde_json::to_string(&payload)
                    .with_context(|_| ErrorKind::PersistentWrite(ACTION_TRANSITION))
                    .map_err(Error::from)
            })
            .transpose()?;
        SQLITE_OPS_COUNT.with_label_values(&["UPDATE"]).inc();
        let _timer = SQLITE_OPS_DURATION
            .with_label_values(&["UPDATE"])
            .start_timer();
        let mut statement = self
            .inner
            .prepare_cached(ACTION_TRANSITION_SQL)
            .with_context(|_| ErrorKind::PersistentWrite(ACTION_TRANSITION))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["UPDATE"]).inc();
                error
            })?;
        statement
            .execute(params![state, state_payload, finished_ts, action_id])
            .with_context(|_| ErrorKind::PersistentWrite(ACTION_TRANSITION))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["UPDATE"]).inc();
                error
            })?;
        self.record_transition(
            action_id,
            state,
            state_payload,
            span.map(|span| span.context().clone()),
        )?;
        Ok(())
    }
}
