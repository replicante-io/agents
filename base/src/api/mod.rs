mod index;
mod info;
mod shards;

// Re-export handlers.
pub use self::index::index;
pub use self::info::AgentInfo;
pub use self::info::DatastoreInfo;
pub use self::shards::Shards;
