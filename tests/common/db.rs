// Database setup helpers for testing

use surrealdb::{Surreal, engine::local::Mem};
use surrealdb::engine::local::Db;
use super::fixtures::TestAccount;

/// Creates an in-memory SurrealDB instance for testing
///
/// This function creates a fresh, isolated database using SurrealDB's memory engine.
/// Each test that calls this function gets its own database instance with no shared state.
///
/// # Examples
///
/// ```ignore
/// #[tokio::test]
/// async fn test_something() {
///     let db = create_test_db().await;
///     // Use db for testing...
/// }
/// ```
pub async fn create_test_db() -> Surreal<Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    db
}

/// Creates test database with migrations applied
///
/// This applies the accounts database migrations to the in-memory database.
/// Use this when your test needs the full database schema.
///
/// Note: For now, this just creates the database. Schema migrations can be
/// added later if tests require them.
///
/// # Examples
///
/// ```ignore
/// #[tokio::test]
/// async fn test_with_schema() {
///     let db = create_test_db_with_migrations().await;
///     // Database now has all tables, indexes, etc.
/// }
/// ```
pub async fn create_test_db_with_migrations() -> Surreal<Db> {
    let db = create_test_db().await;

    // TODO: Apply account resources migrations if needed for tests
    // For now, tests can define their own schema as needed

    db
}

/// Creates test database with a sample account
///
/// This is a convenience function that creates a database with migrations
/// and inserts a test account. Returns both the database and the test account struct.
///
/// # Examples
///
/// ```ignore
/// #[tokio::test]
/// async fn test_with_account() {
///     let (db, account) = create_test_db_with_account("test_account_123").await;
///     // Use db and account for testing...
/// }
/// ```
pub async fn create_test_db_with_account(account_id: &str) -> (Surreal<Db>, TestAccount) {
    let db = create_test_db_with_migrations().await;
    let account = super::fixtures::create_test_account(account_id, "Test Account");

    // Insert account into database
    db.query(format!(
        "CREATE account:{} CONTENT {{ salt: {}, created_at: time::now() }}",
        account.id,
        serde_json::to_string(&account.salt).unwrap()
    ))
    .await
    .unwrap();

    (db, account)
}

/// Returns a shared test accounts database instance
///
/// For tests that need to interact with the accounts database (e.g., testing auth middleware
/// that loads accounts), use this function to get a reference to the test accounts DB.
///
/// Note: This creates a new in-memory instance each time it's called.
/// For true shared state, you'd need to use a lazy_static or once_cell, but
/// the current design prioritizes test isolation over performance.
pub async fn get_test_accounts_db() -> Surreal<Db> {
    create_test_db_with_migrations().await
}
