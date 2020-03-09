use std::cell::RefCell;
use std::collections::btree_map::Entry;
use std::collections::btree_map::IntoIter;
use std::collections::BTreeMap;
use std::panic;
use std::sync::Arc;
use std::sync::Mutex;

use super::Action;

lazy_static::lazy_static! {
    /// Process-global actions register used once registration is complete.
    static ref GLOBAL_ACTIONS: Mutex<Option<ActionsRegister>> = Mutex::new(None);

    /// Process-global actions register for the initial registration phase.
    static ref GLOBAL_REG: Mutex<Option<ActionsRegister>> = {
        Mutex::new(Some(ActionsRegister::default()))
    };
}

thread_local! {
    /// Thread-local actions register to target all "read" `ACTIONS::*` methods.
    ///
    /// The thread-local is initialised with a COPY of the `GLOBAL_REG`
    /// the first time any "read" method is called on the `ACTIONS` register.
    ///
    /// This is also used to allow `ACTIONS::test_with` to set a register for
    /// the duration of a single test without influence from other tests.
    static ACTIVE_REG: RefCell<Option<ActionsRegister>> = RefCell::new(None);
}

/// Ensure a copy of the action register is available as a thread local.
///
/// # Panics
///
///   * If the global actions register is poisoned.
///   * If called while still in the actions registration phase.
fn ensure_thread_register(register: &RefCell<Option<ActionsRegister>>) {
    if register.borrow().is_some() {
        return;
    }
    let actions = GLOBAL_ACTIONS
        .lock()
        .expect("global actions register poisoned")
        .as_ref()
        .expect("attempted to access actions during registration phase")
        .clone();
    *register.borrow_mut() = Some(actions);
}

/// Special interface to a process-global register of actions.
///
/// Both the base agent crate and agent themselves register supported actions during startup.
/// During initialisation of the actions subsystem the register is locked to "read mode".
///
/// Accessing the actions in the register during the registration phase causes a panic.
/// Attempting to register any action after the registration phase is complete panics as well.
/// This ensures a consisten view of the register once reads are needed.
pub struct ACTIONS {}

impl ACTIONS {
    /// Internal method to mark the registration phase as complete.
    ///
    /// # Panics
    /// If the registration phase was already completed.
    pub(crate) fn complete_registration() {
        let global = GLOBAL_REG
            .lock()
            .expect("global actions register is poisoned")
            .take();
        if global.is_none() {
            panic!("global actions registration already complete");
        }
        let mut actions = GLOBAL_ACTIONS
            .lock()
            .expect("global actions register is poisoned");
        if actions.is_some() {
            panic!("global actions registration already complete");
        }
        *actions = global;
    }

    /// Fetch an action from the register.
    ///
    /// # Panics
    ///
    ///   * If the global actions register is poisoned.
    ///   * If called while still in the actions registration phase.
    pub fn get(kind: &str) -> Option<Arc<dyn Action>> {
        ACTIVE_REG.with(|register| {
            ensure_thread_register(&register);
            register.borrow().as_ref().unwrap().get(kind)
        })
    }

    /// Iterate over all registered actions.
    ///
    /// # Panics
    ///
    ///   * If the global actions register is poisoned.
    ///   * If called while still in the actions registration phase.
    pub fn iter() -> Iter {
        ACTIVE_REG.with(|register| {
            ensure_thread_register(&register);
            register.borrow().as_ref().unwrap().iter()
        })
    }

    /// Register an action in the global handler.
    ///
    /// When run in an `ACTIONS::test_with` block the action is registered
    /// in the temporary test register to allow inspection later.
    ///
    /// # Panics
    ///
    ///   * If the global actions register is poisoned.
    ///   * If called after the registration phase is completed.
    ///   * When `ActionsRegister::register` panics.
    pub fn register<A>(action: A)
    where
        A: Action,
    {
        ACTIVE_REG.with(|register| {
            // To support tests, use the thread local if available.
            if register.borrow().is_some() {
                register.borrow_mut().as_mut().unwrap().register(action);
                return;
            }

            // Otherwise register the action with the global registry.
            GLOBAL_REG
                .lock()
                .expect("global actions register poisoned")
                .as_mut()
                .expect("attempted action registration after registration phase is complete")
                .register(action);
        });
    }

    /// Process-global equivalent of `ActionsRegister::register_reserved`.
    #[allow(dead_code)]
    pub(crate) fn register_reserved<A>(action: A)
    where
        A: Action,
    {
        ACTIVE_REG.with(|register| {
            // To support tests, use the thread local if available.
            if register.borrow().is_some() {
                register
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .register_reserved(action);
                return;
            }

            // Otherwise register the action with the global registry.
            GLOBAL_REG
                .lock()
                .expect("global actions register poisoned")
                .as_mut()
                .expect("attempted action registration after registration phase is complete")
                .register_reserved(action);
        });
    }

    /// Process-global equivalent of `ActionsRegister::register_reserved_arc`.
    #[allow(dead_code)]
    pub(crate) fn register_reserved_arc(action: Arc<dyn Action>) {
        ACTIVE_REG.with(|register| {
            // To support tests, use the thread local if available.
            if register.borrow().is_some() {
                register
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .register_reserved_arc(action);
                return;
            }

            // Otherwise register the action with the global registry.
            GLOBAL_REG
                .lock()
                .expect("global actions register poisoned")
                .as_mut()
                .expect("attempted action registration after registration phase is complete")
                .register_reserved_arc(action);
        });
    }

    /// Set the given register as the global register for this call.
    ///
    /// This acts at the thread level so different tests won't interfere with each other.
    /// This method also ensures the original register is restored even on panic.
    #[cfg(any(test, feature = "with_test_support"))]
    pub fn test_with<B>(register: ActionsRegister, block: B) -> ActionsRegister
    where
        B: FnOnce(),
    {
        ACTIVE_REG.with(|local| {
            let original = local.borrow_mut().take();
            *local.borrow_mut() = Some(register);
            let rv = panic::catch_unwind(panic::AssertUnwindSafe(block));
            let register = local.borrow_mut().take();
            *local.borrow_mut() = original;
            match rv {
                Err(error) => panic::resume_unwind(error),
                Ok(rv) => rv,
            }
            register.unwrap()
        })
    }
}

/// Actions are identified by unique names with a fixed format: `SCOPE/ACTION`.
///
///   * `SCOPE` is a DNS like component to ensure uniqueness of IDs.
///   * Any `SCOPE` ending in `replicante.io` is reserved.
///   * `ACTION` should be a descriptive name of what the action does.
///
/// Examples:
///
///   * replicante.io/test/ping
///   * beta.mongodb.org/compact
struct ActionKind<'a> {
    kind: &'a str,
    scope_end: usize,
}

impl<'a> ActionKind<'a> {
    /// Initialise a new ActionKind.
    ///
    /// # Panics
    /// If the given `kind` is not scoped.
    pub fn new(kind: &'a str) -> ActionKind<'a> {
        let scope_end = kind
            .find('/')
            .unwrap_or_else(|| panic!("action kind {} is not scoped", kind));
        ActionKind { kind, scope_end }
    }

    /// TODO
    pub fn is_reserved(&self) -> bool {
        let (scope, _) = self.kind.split_at(self.scope_end);
        scope.ends_with("replicante.io")
    }
}

/// Actions register to store all known actions.
#[derive(Clone)]
pub struct ActionsRegister {
    actions: BTreeMap<String, Arc<dyn Action>>,
}

impl ActionsRegister {
    /// Fetch an action from the register.
    pub fn get(&self, kind: &str) -> Option<Arc<dyn Action>> {
        self.actions.get(kind).cloned()
    }

    /// Iterate over all registered actions.
    pub fn iter(&self) -> Iter {
        Iter(self.actions.clone().into_iter())
    }

    /// Register an action in the register.
    ///
    /// # Panics
    ///
    ///   * If an action with the same Kind is already registered.
    ///   * If the action Kind is not scoped.
    ///   * If the action Kind falls in a reserved scope.
    pub fn register<A>(&mut self, action: A)
    where
        A: Action,
    {
        let kind = action.describe().kind;
        let kind_info = ActionKind::new(&kind);
        if kind_info.is_reserved() {
            panic!("action kind {} is reserved", kind);
        }
        match self.actions.entry(kind) {
            Entry::Vacant(entry) => entry.insert(Arc::new(action)),
            Entry::Occupied(entry) => {
                panic!("action with kind {} is already registered", entry.key())
            }
        };
    }

    /// Same as `ActionsRegister::register` for registration of reserved IDs.
    pub(crate) fn register_reserved<A>(&mut self, action: A)
    where
        A: Action,
    {
        self.register_reserved_arc(Arc::new(action));
    }

    /// Same as `ActionsRegister::register_reserved` for pre-wrapped actions.
    pub(crate) fn register_reserved_arc(&mut self, action: Arc<dyn Action>) {
        let kind = action.describe().kind;
        let kind_info = ActionKind::new(&kind);
        if !kind_info.is_reserved() {
            panic!("action kind {} is NOT reserved", kind);
        }
        match self.actions.entry(kind) {
            Entry::Vacant(entry) => entry.insert(action),
            Entry::Occupied(entry) => {
                panic!("action with kind {} is already registered", entry.key())
            }
        };
    }
}

impl Default for ActionsRegister {
    fn default() -> Self {
        ActionsRegister {
            actions: BTreeMap::new(),
        }
    }
}

/// Iterator over `Action`s in a register.
///
/// Any changes to the original register are not reflected.
pub struct Iter(IntoIter<String, Arc<dyn Action>>);

impl Iterator for Iter {
    type Item = Arc<dyn Action>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, action)| action)
    }
}

#[cfg(test)]
mod tests {
    use opentracingrust::Span;
    use serde_json::Value as Json;

    use super::super::Action;
    use super::super::ActionDescriptor;
    use super::super::ActionRecordView;
    use super::super::ActionValidity;
    use super::ActionsRegister;
    use super::ACTIONS;
    use crate::store::Transaction;
    use crate::Result;

    struct MockAction {}
    impl Action for MockAction {
        fn describe(&self) -> ActionDescriptor {
            ActionDescriptor {
                kind: "test.example.io/mock.action".into(),
                description: "replicante_agent::actions::register::tests::MockAction".into(),
            }
        }

        fn invoke(
            &self,
            _: &mut Transaction,
            _: &dyn ActionRecordView,
            _: Option<&mut Span>,
        ) -> Result<()> {
            panic!("TODO: MockAction::invoke")
        }

        fn validate_args(&self, _: &Json) -> ActionValidity {
            Ok(())
        }
    }

    struct ReservedAction {}
    impl Action for ReservedAction {
        fn describe(&self) -> ActionDescriptor {
            ActionDescriptor {
                kind: "test.replicante.io/mock.action".into(),
                description: "replicante_agent::actions::register::tests::ReservedAction".into(),
            }
        }

        fn invoke(
            &self,
            _: &mut Transaction,
            _: &dyn ActionRecordView,
            _: Option<&mut Span>,
        ) -> Result<()> {
            panic!("TODO: ReservedAction::invoke")
        }

        fn validate_args(&self, _: &Json) -> ActionValidity {
            Ok(())
        }
    }

    struct UnscopedAction {}
    impl Action for UnscopedAction {
        fn describe(&self) -> ActionDescriptor {
            ActionDescriptor {
                kind: "mock".into(),
                description: "replicante_agent::actions::register::tests::UnscopedAction".into(),
            }
        }

        fn invoke(
            &self,
            _: &mut Transaction,
            _: &dyn ActionRecordView,
            _: Option<&mut Span>,
        ) -> Result<()> {
            panic!("TODO: UnscopedAction::invoke")
        }

        fn validate_args(&self, _: &Json) -> ActionValidity {
            Ok(())
        }
    }

    #[test]
    fn get_action() {
        let mut actions = ActionsRegister::default();
        actions.register(MockAction {});
        assert!(
            actions.get("test.example.io/mock.action").is_some(),
            "action not found"
        );
    }

    #[test]
    fn get_action_not_found() {
        let actions = ActionsRegister::default();
        assert!(actions.get("test.mock.action").is_none(), "action found");
    }

    #[test]
    fn iterate_actions() {
        let mut actions = ActionsRegister::default();
        assert!(actions.iter().next().is_none(), "register not empty");
        actions.register(MockAction {});
        actions.register_reserved(ReservedAction {});
        let iter: Vec<String> = actions
            .iter()
            .map(|action| action.describe().kind)
            .collect();
        assert_eq!(
            iter,
            vec![
                "test.example.io/mock.action".to_string(),
                "test.replicante.io/mock.action".to_string()
            ]
        );
    }

    #[test]
    fn register_action() {
        let mut actions = ActionsRegister::default();
        actions.register(MockAction {});
        assert_eq!(actions.actions.len(), 1);
    }

    #[test]
    #[should_panic(expected = "action kind mock is not scoped")]
    fn register_action_fail_unscoped() {
        let mut actions = ActionsRegister::default();
        actions.register(UnscopedAction {});
    }

    #[test]
    #[should_panic(expected = "action kind test.replicante.io/mock.action is reserved")]
    fn register_action_fail_reserved() {
        let mut actions = ActionsRegister::default();
        actions.register(ReservedAction {});
    }

    #[test]
    #[should_panic(expected = "action with kind test.example.io/mock.action is already registered")]
    fn register_action_twice() {
        let mut actions = ActionsRegister::default();
        actions.register(MockAction {});
        actions.register(MockAction {});
    }

    #[test]
    fn register_global_action() {
        let actions = ActionsRegister::default();
        let actions = ACTIONS::test_with(actions, || {
            ACTIONS::register(MockAction {});
        });
        assert_eq!(actions.actions.len(), 1);
    }

    #[test]
    fn register_reserved_action() {
        let mut actions = ActionsRegister::default();
        actions.register_reserved(ReservedAction {});
        assert_eq!(actions.actions.len(), 1);
    }

    #[test]
    #[should_panic(expected = "action kind test.example.io/mock.action is NOT reserved")]
    fn register_action_fail_not_reserved() {
        let mut actions = ActionsRegister::default();
        actions.register_reserved(MockAction {});
    }

    #[test]
    #[should_panic(expected = "action kind mock is not scoped")]
    fn register_reserved_action_fail_unscoped() {
        let mut actions = ActionsRegister::default();
        actions.register_reserved(UnscopedAction {});
    }

    #[test]
    #[should_panic(
        expected = "action with kind test.replicante.io/mock.action is already registered"
    )]
    fn register_reserved_action_twice() {
        let mut actions = ActionsRegister::default();
        actions.register_reserved(ReservedAction {});
        actions.register_reserved(ReservedAction {});
    }

    #[test]
    fn register_reserved_global_action() {
        let actions = ActionsRegister::default();
        let actions = ACTIONS::test_with(actions, || {
            ACTIONS::register_reserved(ReservedAction {});
        });
        assert_eq!(actions.actions.len(), 1);
    }
}
