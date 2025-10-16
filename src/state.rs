use std::sync::Arc;

use async_trait::async_trait;
use tracing::instrument;

use crate::{
    auth::AuthProvider,
    db::{DBConnection, accounts_db, resources_db},
    env::Env,
};
use archodex_error::anyhow;

/// State-based dependency injection for testability
///
/// AppState holds the shared application state including database connections,
/// factories for creating per-request resources, and authentication providers.
/// This enables tests to inject in-memory databases and fixed authentication
/// while production code uses global connection pools and real validation.
#[derive(Clone)]
pub struct AppState {
    /// Factory for creating per-account resources database connections
    pub resources_db_factory: Arc<dyn ResourcesDbFactory + Send + Sync>,
    /// Authentication provider for validating requests
    pub auth_provider: Arc<dyn AuthProvider>,
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
        // Get the appropriate URL based on configuration
        #[cfg(feature = "archodex-com")]
        let url = service_url
            .ok_or_else(|| anyhow::anyhow!("service_url is required for archodex-com feature"))?;

        #[cfg(not(feature = "archodex-com"))]
        let url = service_url.unwrap_or_else(|| Env::surrealdb_url());

        resources_db(url, account_id).await
    }
}
