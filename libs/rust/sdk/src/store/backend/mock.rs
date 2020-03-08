use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use opentracingrust::SpanContext;
use serde_json::Value as Json;

use crate::actions::ActionHistoryItem;
use crate::actions::ActionRecord;
use crate::actions::ActionState;
use crate::store::interface::ActionImpl;
use crate::store::interface::ActionInterface;
use crate::store::interface::ActionsImpl;
use crate::store::interface::ConnectionImpl;
use crate::store::interface::ConnectionInterface;
use crate::store::interface::StoreInterface;
use crate::store::interface::TransactionImpl;
use crate::store::interface::TransactionInterface;
use crate::store::Iter;
use crate::Result;

#[derive(Clone)]
struct MockState {
    actions: HashMap<String, ActionRecord>,
    actions_queue: VecDeque<String>,
}

impl Default for MockState {
    fn default() -> Self {
        MockState {
            actions: HashMap::new(),
            actions_queue: VecDeque::new(),
        }
    }
}

type SyncState = Arc<Mutex<MockState>>;

/// Mocked store for tests.
pub struct MockStore {
    state: SyncState,
}

impl MockStore {
    pub fn new() -> MockStore {
        let state = Arc::new(Mutex::new(MockState::default()));
        MockStore { state }
    }
}

impl StoreInterface for MockStore {
    fn connection(&self) -> Result<ConnectionImpl> {
        let connection = ConnectionImpl::new(Connection {
            state: self.state.clone(),
        });
        Ok(connection)
    }

    fn migrate(&self) -> Result<()> {
        Ok(())
    }
}

struct Connection {
    state: SyncState,
}

impl ConnectionInterface for Connection {
    fn transaction(&mut self) -> Result<TransactionImpl> {
        // Global state is outside the tx.
        // State is tx copy to be modified.
        // On commit the tx copy is made global.
        let global = self.state.clone();
        let state: MockState = self.state.lock().unwrap().clone();
        let state = Arc::new(Mutex::new(state));
        let transaction = TransactionImpl::new(Transaction { global, state });
        Ok(transaction)
    }
}

struct Transaction {
    global: SyncState,
    state: SyncState,
}

impl TransactionInterface for Transaction {
    /// Access single action query interface.
    fn action(&mut self) -> ActionImpl {
        ActionImpl::new(Action {
            state: self.state.clone(),
        })
    }

    /// Access the actions query interface.
    fn actions(&mut self) -> ActionsImpl {
        panic!("TODO: MockStore::Transaction::actions")
    }

    /// Commit and invalidate the transaction.
    fn commit(&mut self) -> Result<()> {
        let state = self.state.lock().unwrap().clone();
        *self.global.lock().unwrap() = state;
        Ok(())
    }

    /// Rollback and invalidate the transaction.
    fn rollback(&mut self) -> Result<()> {
        // Rollbacks are no-ops.
        Ok(())
    }
}

struct Action {
    state: SyncState,
}

impl ActionInterface for Action {
    fn get(&self, id: &str, _: Option<SpanContext>) -> Result<Option<ActionRecord>> {
        let state = self.state.lock().unwrap();
        let action = state.actions.get(id).cloned();
        Ok(action)
    }

    fn history(&self, _id: &str, _: Option<SpanContext>) -> Result<Iter<ActionHistoryItem>> {
        panic!("TODO: MockStore::action::history")
    }

    fn insert(&self, action: ActionRecord, _: Option<SpanContext>) -> Result<()> {
        let id = action.id;
        let mut state = self.state.lock().unwrap();
        state.actions.insert(id.to_string(), action);
        state.actions_queue.push_back(id.to_string());
        Ok(())
    }

    fn next(&self, _: Option<SpanContext>) -> Result<Option<ActionRecord>> {
        let mut state = self.state.lock().unwrap();
        let next = state
            .actions_queue
            .pop_front()
            .and_then(|id| state.actions.get(&id))
            .cloned();
        Ok(next)
    }

    fn transition(
        &self,
        action: &ActionRecord,
        transition_to: ActionState,
        payload: Option<Json>,
        _: Option<SpanContext>,
    ) -> Result<()> {
        let id = action.id.to_string();
        let state_finished = transition_to.is_finished();
        let mut state = self.state.lock().unwrap();
        let record = state.actions.get_mut(&id).unwrap();
        record.set_state(transition_to);
        record.set_state_payload(payload);
        let finished = state
            .actions_queue
            .front()
            .map(|front| *front == id)
            .unwrap_or(false);
        if finished && state_finished {
            state.actions_queue.pop_front();
        }
        Ok(())
    }
}
