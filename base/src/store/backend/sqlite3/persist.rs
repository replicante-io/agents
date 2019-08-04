use failure::ResultExt;
use opentracingrust::SpanContext;
use opentracingrust::StartOptions;
use rusqlite::params;

use replicante_util_tracing::MaybeTracer;

use crate::actions::ActionRecord;
use crate::metrics::SQLITE_OPS_COUNT;
use crate::metrics::SQLITE_OPS_DURATION;
use crate::metrics::SQLITE_OP_ERRORS_COUNT;
use crate::store::interface::PersistInterface;
use crate::ErrorKind;
use crate::Result;

const PERSIST_ACTION: &str = "persist.action";
const PERSIST_ACTION_SQL: &str = r#"
INSERT INTO actions (
    action,
    agent_version,
    args,
    created_ts,
    headers,
    id,
    requester,
    state
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?);
"#;

pub struct Persist<'a, 'b: 'a> {
    inner: &'a rusqlite::Transaction<'b>,
    tracer: MaybeTracer,
}

impl<'a, 'b: 'a> Persist<'a, 'b> {
    pub fn new(inner: &'a rusqlite::Transaction<'b>, tracer: MaybeTracer) -> Persist<'a, 'b> {
        Persist { inner, tracer }
    }
}

impl<'a, 'b: 'a> PersistInterface for Persist<'a, 'b> {
    fn action(&self, action: ActionRecord, span: Option<SpanContext>) -> Result<()> {
        let _span = self.tracer.with(|tracer| {
            let mut opts = StartOptions::default();
            if let Some(context) = span {
                opts = opts.child_of(context);
            }
            let mut span = tracer.span_with_options("store.sqlite.insert", opts);
            span.tag("sql", PERSIST_ACTION_SQL);
            span.auto_finish()
        });
        let args = serde_json::to_string(&action.args)
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))?;
        let headers = serde_json::to_string(&action.headers)
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))?;
        let requester = serde_json::to_string(&action.requester)
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))?;
        let state = serde_json::to_string(&action.state)
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))?;
        SQLITE_OPS_COUNT.with_label_values(&["INSERT"]).inc();
        let _timer = SQLITE_OPS_DURATION
            .with_label_values(&["INSERT"])
            .start_timer();
        let mut statement = self
            .inner
            .prepare_cached(PERSIST_ACTION_SQL)
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))
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
                action.id.to_string(),
                requester,
                state,
            ])
            .with_context(|_| ErrorKind::PersistentWrite(PERSIST_ACTION))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["INSERT"]).inc();
                error
            })?;
        Ok(())
    }
}
