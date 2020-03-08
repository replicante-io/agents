use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use opentracingrust::SpanContext;
use serde_json::Value as Json;

use super::Iter;
use crate::actions::ActionHistoryItem;
use crate::actions::ActionListItem;
use crate::actions::ActionRecord;
use crate::actions::ActionState;
use crate::Result;

// Macro definition to generate an interface trait with a wrapping wrapper
// for dynamic dispatch to Send + Sync + 'static implementations.
macro_rules! arc_interface {
    (
        $(#[$struct_meta:meta])*
        struct $struct_name:ident,
        $(#[$trait_meta:meta])*
        trait $trait_name:ident,
        interface $trait_def:tt
    ) => {
        $(#[$trait_meta])*
        pub trait $trait_name: Send + Sync $trait_def

        $(#[$struct_meta])*
        pub struct $struct_name(Arc<dyn $trait_name>);

        impl $struct_name {
            pub fn new<I: $trait_name + 'static>(interface: I) -> Self {
                Self(Arc::new(interface))
            }
        }

        impl Deref for $struct_name {
            type Target = dyn $trait_name + 'static;
            fn deref(&self) -> &(dyn $trait_name + 'static) {
                self.0.deref()
            }
        }
    }
}
macro_rules! box_interface {
    // 'static lifetime
    (
        $(#[$struct_meta:meta])*
        struct $struct_name:ident,
        $(#[$trait_meta:meta])*
        trait $trait_name:ident,
        interface $trait_def:tt
    ) => {
        $(#[$trait_meta])*
        pub trait $trait_name $trait_def

        $(#[$struct_meta])*
        pub struct $struct_name(Box<dyn $trait_name>);

        impl $struct_name {
            pub fn new<I: $trait_name + 'static>(interface: I) -> Self {
                Self(Box::new(interface))
            }
        }

        impl Deref for $struct_name {
            type Target = dyn $trait_name + 'static;
            fn deref(&self) -> &(dyn $trait_name + 'static) {
                self.0.deref()
            }
        }

        impl DerefMut for $struct_name {
            fn deref_mut(&mut self) -> &mut (dyn $trait_name + 'static) {
                self.0.deref_mut()
            }
        }
    };
    // generic 'a lifetime
    (
        lifetime $lifetime:lifetime,
        $(#[$struct_meta:meta])*
        struct $struct_name:ident,
        $(#[$trait_meta:meta])*
        trait $trait_name:ident,
        interface $trait_def:tt
    ) => {
        $(#[$trait_meta])*
        pub trait $trait_name $trait_def

        $(#[$struct_meta])*
        pub struct $struct_name<$lifetime>(Box<dyn $trait_name + $lifetime>);

        impl<$lifetime> $struct_name<$lifetime> {
            pub fn new<I: $trait_name + $lifetime>(interface: I) -> Self {
                Self(Box::new(interface))
            }
        }

        impl<$lifetime> Deref for $struct_name<$lifetime> {
            type Target = dyn $trait_name + $lifetime;
            fn deref(&self) -> &(dyn $trait_name + $lifetime) {
                self.0.deref()
            }
        }

        impl<$lifetime> DerefMut for $struct_name<$lifetime> {
            fn deref_mut(&mut self) -> &mut (dyn $trait_name + $lifetime) {
                self.0.deref_mut()
            }
        }
    };
}

arc_interface! {
    /// Dynamic dispatch all operations to a backend-specific implementation.
    #[derive(Clone)]
    struct StoreImpl,

    /// Definition of top level store operations.
    ///
    /// See `crate::store::Store` for descriptions of methods.
    trait StoreInterface,

    interface {
        /// Request a new connection to the store.
        fn connection(&self) -> Result<ConnectionImpl>;

        /// Perform database initialisation and applies migrations.
        fn migrate(&self) -> Result<()>;
    }
}

box_interface! {
    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct ConnectionImpl,

    /// Operations on DB connections.
    trait ConnectionInterface,

    interface {
        /// Start a new transaction and return a manager for it.
        fn transaction(&mut self) -> Result<TransactionImpl>;
    }
}

box_interface! {
    lifetime 'a,

    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct ActionImpl,

    /// Interface to fetch actions from the store.
    trait ActionInterface,

    interface {
        /// Fetch an action record by ID.
        fn get(&self, id: &str, span: Option<SpanContext>) -> Result<Option<ActionRecord>>;

        /// Fetch an action record's transition history.
        fn history(
            &self,
            id: &str,
            span: Option<SpanContext>,
        ) -> Result<Iter<ActionHistoryItem>>;

        /// Persist a NEW action to the store.
        fn insert(&self, action: ActionRecord, span: Option<SpanContext>) -> Result<()>;

        /// Fetch the next RUNNING or NEW action.
        fn next(&self, span: Option<SpanContext>) -> Result<Option<ActionRecord>>;

        /// Transition the action to a new state.
        fn transition(
            &self,
            action: &ActionRecord,
            transition_to: ActionState,
            payload: Option<Json>,
            span: Option<SpanContext>,
        ) -> Result<()>;
    }
}

box_interface! {
    lifetime 'a,

    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct ActionsImpl,

    /// Interface to fetch actions from the store.
    trait ActionsInterface,

    interface {
        /// Iterate over the most recent 100 finished actions, newest action first.
        fn finished(&self, span: Option<SpanContext>) -> Result<Iter<ActionListItem>>;

        /// Iterate over running and pending actions, oldest action first.
        fn queue(&self, span: Option<SpanContext>) -> Result<Iter<ActionListItem>>;

        /// Prune finished historic actions to prevent endless DB growth.
        fn prune(&self, keep: u32, limit: u32, span: Option<SpanContext>) -> Result<()>;
    }
}

box_interface! {
    lifetime 'a,

    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct TransactionImpl,

    /// Interface to transactional operations on the store.
    trait TransactionInterface,

    interface {
        /// Access single action query interface.
        fn action(&mut self) -> ActionImpl;

        /// Access the actions query interface.
        fn actions(&mut self) -> ActionsImpl;

        /// Commit and invalidate the transaction.
        fn commit(&mut self) -> Result<()>;

        /// Rollback and invalidate the transaction.
        fn rollback(&mut self) -> Result<()>;
    }
}
