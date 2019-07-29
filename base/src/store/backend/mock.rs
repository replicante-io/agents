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
    fn migrate(&self) -> Result<()> {
        Ok(())
    }
}
