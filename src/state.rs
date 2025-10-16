use std::sync::Arc;

use async_trait::async_trait;
use tracing::instrument;

use crate::{db::{DBConnection, accounts_db, resources_db}, env::Env};
use archodex_error::anyhow;

/// State-based dependency injection for testability
///
/// AppState holds the shared application state including database connections
/// and factories for creating per-request resources. This enables tests to
/// inject in-memory databases while production code uses global connection pools.
#[derive(Clone)]
pub struct AppState {
    /// Factory for creating per-account resources database connections
    pub resources_db_factory: Arc<dyn ResourcesDbFactory + Send + Sync>,
}

/// Factory trait for creating accounts and resources database connections
///
/// This trait abstracts database connection creation, allowing production code
/// to use global connection pooling while tests can inject in-memory databases.
///
/// # Parameters
/// - `account_id`: The account ID for namespace selection
/// - `service_url`: Optional custom SurrealDB URL (None uses default from environment)
#[async_trait]
pub trait ResourcesDbFactory {
    async fn create_accounts_connection(&self) -> anyhow::Result<DBConnection>;

    async fn create_resources_connection(
        &self,
        account_id: &str,
        service_url: Option<&str>,
    ) -> anyhow::Result<DBConnection>;
}

/// Production implementation using global connection pool
///
/// This factory uses the existing `accounts_db()` and `resources_db()` functions which maintain
/// global connection pools for performance. This provides zero overhead
/// compared to direct calls to these functions.
pub struct GlobalResourcesDbFactory;

#[async_trait]
impl ResourcesDbFactory for GlobalResourcesDbFactory {
    #[instrument(err, skip(self))]
    async fn create_accounts_connection(&self) -> anyhow::Result<DBConnection> {
        accounts_db().await.map_err(|e| anyhow::anyhow!(e))
    }

    #[instrument(err, skip(self))]
    async fn create_resources_connection(
        &self,
        account_id: &str,
        service_url: Option<&str>,
    ) -> anyhow::Result<DBConnection> {
        let url = service_url.unwrap_or_else(|| Env::surrealdb_url());
        resources_db(url, account_id).await
    }
}
