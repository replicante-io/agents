use crate::store::interface::ConnectionImpl;
use crate::store::interface::StoreInterface;
use crate::Result;

/// Mocked store for tests.
pub struct MockStore {}

impl MockStore {
    pub fn new() -> MockStore {
        MockStore {}
    }
}

impl StoreInterface for MockStore {
    fn connection(&self) -> Result<ConnectionImpl> {
        panic!("NotImplemented: MockStore::connection")
    }

    fn migrate(&self) -> Result<()> {
        Ok(())
    }
}
