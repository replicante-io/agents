use std::str::FromStr;

use chrono::TimeZone;
use chrono::Utc;
use failure::ResultExt;
use opentracingrust::SpanContext;
use opentracingrust::StartOptions;
use rusqlite::params;
use uuid::Uuid;

use replicante_util_tracing::MaybeTracer;

use crate::actions::ActionRecord;
use crate::metrics::SQLITE_OPS_COUNT;
use crate::metrics::SQLITE_OPS_DURATION;
use crate::metrics::SQLITE_OP_ERRORS_COUNT;
use crate::store::interface::ActionInterface;
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
    state
FROM actions
WHERE id = ?;
"#;

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

pub struct Action<'a, 'b: 'a> {
    inner: &'a rusqlite::Transaction<'b>,
    tracer: MaybeTracer,
}

impl<'a, 'b: 'a> Action<'a, 'b> {
    pub fn new(inner: &'a rusqlite::Transaction<'b>, tracer: MaybeTracer) -> Action<'a, 'b> {
        Action { inner, tracer }
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
        let id: String = decode_or_return!(row.get("id"), ACTION_GET);
        let id = decode_or_return!(Uuid::from_str(&id), ACTION_GET);
        let action: String = decode_or_return!(row.get("action"), ACTION_GET);
        let agent_version: String = decode_or_return!(row.get("agent_version"), ACTION_GET);
        let args: String = decode_or_return!(row.get("args"), ACTION_GET);
        let args = decode_or_return!(serde_json::from_str(&args), ACTION_GET);
        let created_ts: i64 = decode_or_return!(row.get("created_ts"), ACTION_GET);
        let created_ts = Utc.timestamp(created_ts, 0);
        let headers: String = decode_or_return!(row.get("headers"), ACTION_GET);
        let headers = decode_or_return!(serde_json::from_str(&headers), ACTION_GET);
        let requester: String = decode_or_return!(row.get("requester"), ACTION_GET);
        let requester = decode_or_return!(serde_json::from_str(&requester), ACTION_GET);
        let state: String = decode_or_return!(row.get("state"), ACTION_GET);
        let state = decode_or_return!(serde_json::from_str(&state), ACTION_GET);
        Ok(Some(ActionRecord {
            action,
            agent_version,
            args,
            created_ts,
            headers,
            id,
            requester,
            state,
        }))
    }
}
