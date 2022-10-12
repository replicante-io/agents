use std::str::FromStr;

use failure::ResultExt;
use opentracingrust::SpanContext;
use opentracingrust::StartOptions;
use rusqlite::params;
use rusqlite::Statement;
use uuid::Uuid;

use replicante_util_tracing::MaybeTracer;

use crate::actions::ActionListItem;
use crate::actions::ActionState;
use crate::metrics::SQLITE_OPS_COUNT;
use crate::metrics::SQLITE_OPS_DURATION;
use crate::metrics::SQLITE_OP_ERRORS_COUNT;
use crate::store::interface::ActionsInterface;
use crate::store::Iter;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

const ACTIONS_FINISHED: &str = "action.finished";
const ACTIONS_FINISHED_SQL: &str = r#"
SELECT
    kind, id, state
FROM actions
WHERE finished_ts IS NOT NULL
ORDER BY scheduled_ts DESC, ROWID DESC
-- Limit result as a form of blast radius containment from bugs or overload.
-- There really should not be many finished actions still on the agent DB.
LIMIT 100;
"#;
const ACTIONS_QUEUE: &str = "action.queue";
const ACTIONS_QUEUE_SQL: &str = r#"
SELECT
    kind, id, state
FROM actions
WHERE finished_ts IS NULL
ORDER BY scheduled_ts ASC, ROWID ASC
-- Limit result as a form of blast radius containment in case of bugs.
-- There really should not be many running/pending actions on an agent.
LIMIT 100;
"#;
const ACTIONS_PRUNE: &str = "action.prune";
const ACTIONS_PRUNE_SQL: &str = r#"
DELETE FROM actions
WHERE id IN (
    SELECT id
    FROM actions
    WHERE finished_ts IS NOT NULL
    ORDER BY finished_ts DESC
    -- Limit result as a form of blast radius containment in case of bugs.
    -- There really should not be many finished actions to clean up on an agent.
    LIMIT ?1
    -- Keep some history in the DB for sync with Core.
    OFFSET ?2
);
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

/// Helper to convert the result of a SELECT id, state ...; into an ActionListItem iterator.
fn parse_actions_list(statement: &mut Statement, op: &'static str) -> Result<Iter<ActionListItem>> {
    let mut results = Vec::new();
    let mut rows = statement
        .query([])
        .with_context(|_| ErrorKind::PersistentRead(op))?;
    let mut maybe_row = rows
        .next()
        .with_context(|_| ErrorKind::PersistentRead(op))?;
    while let Some(row) = maybe_row {
        let id: String = decode_or_continue!(row.get("id"), results, op);
        let id = decode_or_continue!(Uuid::from_str(&id), results, op);
        let kind: String = decode_or_continue!(row.get("kind"), results, op);
        let state: String = decode_or_continue!(row.get("state"), results, op);
        let state: ActionState = decode_or_continue!(serde_json::from_str(&state), results, op);
        results.push(Ok(ActionListItem { kind, id, state }));
        maybe_row = rows
            .next()
            .with_context(|_| ErrorKind::PersistentRead(op))?;
    }
    Ok(Iter::new(results.into_iter()))
}

pub struct Actions<'a, 'b: 'a> {
    inner: &'a rusqlite::Transaction<'b>,
    tracer: MaybeTracer,
}

impl<'a, 'b: 'a> Actions<'a, 'b> {
    pub fn new(inner: &'a rusqlite::Transaction<'b>, tracer: MaybeTracer) -> Actions<'a, 'b> {
        Actions { inner, tracer }
    }
}

impl<'a, 'b: 'a> ActionsInterface for Actions<'a, 'b> {
    fn finished(&self, span: Option<SpanContext>) -> Result<Iter<ActionListItem>> {
        let _span = self.tracer.with(|tracer| {
            let mut opts = StartOptions::default();
            if let Some(context) = span {
                opts = opts.child_of(context);
            }
            let mut span = tracer.span_with_options("store.sqlite.select", opts);
            span.tag("sql", ACTIONS_FINISHED_SQL);
            span.auto_finish()
        });
        SQLITE_OPS_COUNT.with_label_values(&["SELECT"]).inc();
        let _timer = SQLITE_OPS_DURATION
            .with_label_values(&["SELECT"])
            .start_timer();
        let mut statement = self
            .inner
            .prepare_cached(ACTIONS_FINISHED_SQL)
            .with_context(|_| ErrorKind::PersistentRead(ACTIONS_FINISHED))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["SELECT"]).inc();
                error
            })?;
        parse_actions_list(&mut statement, ACTIONS_FINISHED).map_err(|error| {
            SQLITE_OP_ERRORS_COUNT.with_label_values(&["SELECT"]).inc();
            error
        })
    }

    fn queue(&self, span: Option<SpanContext>) -> Result<Iter<ActionListItem>> {
        let _span = self.tracer.with(|tracer| {
            let mut opts = StartOptions::default();
            if let Some(context) = span {
                opts = opts.child_of(context);
            }
            let mut span = tracer.span_with_options("store.sqlite.select", opts);
            span.tag("sql", ACTIONS_QUEUE_SQL);
            span.auto_finish()
        });
        SQLITE_OPS_COUNT.with_label_values(&["SELECT"]).inc();
        let _timer = SQLITE_OPS_DURATION
            .with_label_values(&["SELECT"])
            .start_timer();
        let mut statement = self
            .inner
            .prepare_cached(ACTIONS_QUEUE_SQL)
            .with_context(|_| ErrorKind::PersistentRead(ACTIONS_QUEUE))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["SELECT"]).inc();
                error
            })?;
        parse_actions_list(&mut statement, ACTIONS_QUEUE).map_err(|error| {
            SQLITE_OP_ERRORS_COUNT.with_label_values(&["SELECT"]).inc();
            error
        })
    }

    fn prune(&self, keep: u32, limit: u32, span: Option<SpanContext>) -> Result<()> {
        let _span = self.tracer.with(|tracer| {
            let mut opts = StartOptions::default();
            if let Some(context) = span {
                opts = opts.child_of(context);
            }
            let mut span = tracer.span_with_options("store.sqlite.delete", opts);
            span.tag("sql", ACTIONS_PRUNE_SQL);
            span.auto_finish()
        });
        SQLITE_OPS_COUNT.with_label_values(&["DELETE"]).inc();
        let _timer = SQLITE_OPS_DURATION
            .with_label_values(&["DELETE"])
            .start_timer();
        let mut statement = self
            .inner
            .prepare_cached(ACTIONS_PRUNE_SQL)
            .with_context(|_| ErrorKind::PersistentWrite(ACTIONS_PRUNE))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["DELETE"]).inc();
                error
            })?;
        statement
            .execute(params![limit, keep])
            .with_context(|_| ErrorKind::PersistentWrite(ACTIONS_PRUNE))
            .map_err(|error| {
                SQLITE_OP_ERRORS_COUNT.with_label_values(&["DELETE"]).inc();
                error
            })?;
        Ok(())
    }
}
