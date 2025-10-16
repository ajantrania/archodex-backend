// Shared test helpers for Archodex backend testing
//
// This module provides helper functions for:
// - Database setup (in-memory SurrealDB instances)
// - Test data generation (fixtures for accounts, resources, events, reports)
// - Test routers (HTTP testing with auth bypass)

pub mod db;
pub mod fixtures;
pub mod test_router;

pub use db::*;
pub use fixtures::*;
pub use test_router::*;

/// Sets up environment variables required for testing
pub fn setup_test_env() {
    // Set required environment variables for test mode
    // SAFETY: This is only called in tests, and we're setting string values
    unsafe {
        std::env::set_var("ARCHODEX_DOMAIN", "test.archodex.com");
        std::env::set_var("ARCHODEX_TEST_MODE", "1");
    }
}
