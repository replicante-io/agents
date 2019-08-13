use opentracingrust::SpanContext;
use serde_json::Value as Json;
use slog::Logger;

use replicante_util_failure::capture_fail;
use replicante_util_failure::failure_info;

mod backend;
mod interface;

pub use self::backend::backend_factory;

use self::interface::StoreImpl;
use self::interface::TransactionImpl;
use crate::actions::ensure_transition_allowed;
use crate::actions::ActionListItem;
use crate::actions::ActionRecord;
use crate::actions::ActionRecordHistory;
use crate::actions::ActionState;
use crate::Result;

/// Single Action query interface.
pub struct Action<'a> {
    inner: self::interface::ActionImpl<'a>,
}

impl<'a> Action<'a> {
    /// Fetch an action record by ID.
    pub fn get<S>(&self, id: &str, span: S) -> Result<Option<ActionRecord>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.inner.get(id, span.into())
    }

    /// Fetch an action record's transition history.
    pub fn history<S>(&self, id: &str, span: S) -> Result<Iter<ActionRecordHistory>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.inner.history(id, span.into())
    }

    /// Persist a NEW action to the store.
    pub fn insert<S>(&self, action: ActionRecord, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.inner.insert(action, span.into())
    }

    /// Fetch the next RUNNING or NEW action.
    pub fn next<S>(&self, span: S) -> Result<Option<ActionRecord>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.inner.next(span.into())
    }

    /// Transition the action to a new state.
    ///
    /// # Allowed transitions
    /// Actions cannot transition from one arbitrary state to another.
    /// Restrictions apply to ensure that actions can only transition from
    /// one state to another if it makes logical sense for the transition.
    ///
    /// For example, and action can transtion from `New` to `Failed` if
    /// the action is invalid or cannot be otherwise executed.
    /// On the other hand an action cannot go from `Failed` to `New`.
    ///
    /// For a diagram of valid state transitions see
    /// `docs/docs/assets/action-states.dot`.
    ///
    /// # Panics
    /// If the state transition is not allowd this method panics.
    pub fn transition<P, S>(
        &self,
        action: &ActionRecord,
        transition_to: ActionState,
        payload: P,
        span: S,
    ) -> Result<()>
    where
        P: Into<Option<Json>>,
        S: Into<Option<SpanContext>>,
    {
        ensure_transition_allowed(&action.state, &transition_to);
        self.inner
            .transition(action, transition_to, payload.into(), span.into())
    }
}

/// Actions query interface.
pub struct Actions<'a> {
    inner: self::interface::ActionsImpl<'a>,
}

impl<'a> Actions<'a> {
    /// Iterate over the most recent 100 finished actions.
    pub fn finished<S>(&self, span: S) -> Result<Iter<ActionListItem>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.inner.finished(span.into())
    }

    /// Iterate over running and pending actions.
    pub fn queue<S>(&self, span: S) -> Result<Iter<ActionListItem>>
    where
        S: Into<Option<SpanContext>>,
    {
        self.inner.queue(span.into())
    }

    /// Prune finished historic actions to prevent endless DB growth.
    pub fn prune<S>(&self, keep: u32, limit: u32, span: S) -> Result<()>
    where
        S: Into<Option<SpanContext>>,
    {
        self.inner.prune(keep, limit, span.into())
    }
}

/// Iterator over store results.
pub struct Iter<T>(Box<dyn Iterator<Item = Result<T>>>);

impl<T> Iter<T> {
    fn new<I>(iter: I) -> Iter<T>
    where
        I: Iterator<Item = Result<T>> + 'static,
    {
        Iter(Box::new(iter))
    }
}

impl<T> Iterator for Iter<T> {
    type Item = Result<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Interface to the agent's persistent storage.
#[derive(Clone)]
pub struct Store {
    logger: Logger,
    inner: StoreImpl,
}

impl Store {
    /// Perform database initialisation and applies migrations.
    ///
    /// This method requires a mutable borrow to ensure it can only
    /// be called during the process initialisation phase.
    pub fn migrate(&mut self) -> Result<()> {
        self.inner.migrate()
    }

    #[cfg(any(test, feature = "with_test_support"))]
    pub fn mock() -> Store {
        let inner = self::backend::mock::MockStore::new();
        let inner = StoreImpl::new(inner);
        let logger = Logger::root(slog::Discard, slog::o!());
        Store { inner, logger }
    }

    pub fn with_transaction<F, T>(&self, block: F) -> Result<T>
    where
        F: FnOnce(&mut Transaction) -> Result<T>,
    {
        let mut connection = self.inner.connection()?;
        let tx = connection.transaction()?;
        let mut tx = Transaction { inner: tx };
        match block(&mut tx) {
            Err(error) => {
                if let Err(error) = tx.rollback() {
                    capture_fail!(
                        &error,
                        self.logger,
                        "Failed to rollback failed transaction";
                        failure_info(&error),
                    );
                }
                Err(error)
            }
            Ok(rv) => {
                tx.commit()?;
                Ok(rv)
            }
        }
    }
}

/// Interface to transactional operations on the store.
pub struct Transaction<'a> {
    inner: TransactionImpl<'a>,
}

impl<'a> Transaction<'a> {
    /// Access single action query interface.
    pub fn action(&mut self) -> Action {
        let inner = self.inner.action();
        Action { inner }
    }

    /// Access the actions query interface.
    pub fn actions(&mut self) -> Actions {
        let inner = self.inner.actions();
        Actions { inner }
    }

    /// Commit and consume the transaction.
    pub fn commit(mut self) -> Result<()> {
        self.inner.commit()
    }

    /// Rollback and consume the transaction.
    pub fn rollback(mut self) -> Result<()> {
        self.inner.rollback()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Store;
    use crate::actions::ActionRecord;
    use crate::actions::ActionRequester;
    use crate::actions::ActionState;

    #[test]
    #[should_panic(expected = "actions are not allowed to transition from New to Cancelled")]
    fn transition_forbidden() {
        let record = ActionRecord::new("test", json!(null), ActionRequester::Api);
        let store = Store::mock();
        store
            .with_transaction(|tx| {
                tx.action().insert(record.clone(), None)?;
                tx.action()
                    .transition(&record, ActionState::Cancelled, None, None)
            })
            .unwrap();
    }

    #[test]
    fn transition_success() {
        let record = ActionRecord::new("test", json!(null), ActionRequester::Api);
        let store = Store::mock();
        store
            .with_transaction(|tx| {
                tx.action().insert(record.clone(), None)?;
                tx.action()
                    .transition(&record, ActionState::Done, None, None)
            })
            .unwrap();
    }
}
