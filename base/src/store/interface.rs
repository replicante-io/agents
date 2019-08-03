use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use super::Iter;
use crate::actions::ActionListItem;
use crate::actions::ActionRecord;
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
        fn get(&self, id: &str) -> Result<Option<ActionRecord>>;
    }
}

box_interface! {
    lifetime 'a,

    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct ActionsImpl,

    /// Interface to fetch actions from the store.
    trait ActionsInterface,

    interface {
        /// Iterate over the most recent 100 finished actions.
        fn finished(&self) -> Result<Iter<ActionListItem>>;

        /// Iterate over running and pending actions.
        fn queue(&self) -> Result<Iter<ActionListItem>>;
    }
}

box_interface! {
    lifetime 'a,

    /// Dynamic dispatch all operations to a backend-specific implementation.
    struct PersistImpl,

    /// Interface to persist data to the store.
    trait PersistInterface,

    interface {
        /// Persist a NEW action to the store.
        fn action(&self, action: ActionRecord) -> Result<()>;
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

        /// Access the interface to persist data to the store.
        fn persist(&mut self) -> PersistImpl;

        /// Rollback and invalidate the transaction.
        fn rollback(&mut self) -> Result<()>;
    }
}
