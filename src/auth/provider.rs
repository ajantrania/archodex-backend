use async_trait::async_trait;
use axum::http::{HeaderMap, header::AUTHORIZATION};
use tracing::instrument;

use archodex_error::anyhow::{self, Context as _};

use crate::report_api_key::ReportApiKey;

/// Authentication context returned after successful authentication
///
/// This contains the authenticated account ID and key ID that were validated
/// from the request. Matches the tuple returned by ReportApiKey::validate_value.
#[derive(Clone, Debug)]
pub struct AuthContext {
    pub account_id: String,
    pub key_id: u32,
}

/// Pluggable authentication provider for production and testing
///
/// This trait abstracts authentication logic, allowing production code to use
/// real JWT/API key validation while tests can inject fixed authentication
/// contexts without requiring valid tokens.
///
/// # Production Implementation
/// `RealAuthProvider` extracts the Authorization header and calls the existing
/// `ReportApiKey::validate_value()` function to perform cryptographic validation.
///
/// # Test Implementation
/// `FixedAuthProvider` returns a pre-configured `AuthContext` without any
/// validation, enabling integration tests to bypass authentication.
#[async_trait]
pub trait AuthProvider: Send + Sync + 'static {
    /// Authenticate a request and return the authentication context
    ///
    /// # Parameters
    /// - `headers`: The HTTP request headers containing authentication credentials
    ///
    /// # Returns
    /// - `Ok(AuthContext)`: Authentication successful with account and key IDs
    /// - `Err`: Authentication failed (missing header, invalid token, etc.)
    async fn authenticate(&self, headers: &HeaderMap) -> anyhow::Result<AuthContext>;
}

/// Production authentication implementation using JWT/API key validation
///
/// This implementation uses the adapter pattern to reuse existing validation logic.
/// It extracts the Authorization header from the request and calls
/// `ReportApiKey::validate_value()` to perform the actual cryptographic validation.
///
/// # Design Pattern
/// This is a thin adapter over existing validation logic, not a reimplementation.
/// The cryptographic validation, protobuf decoding, KMS operations, and nonce
/// validation all remain in the existing `ReportApiKey::validate_value()` function.
#[derive(Clone)]
pub struct RealAuthProvider;

#[async_trait]
impl AuthProvider for RealAuthProvider {
    /// Authenticate using real API key validation
    ///
    /// Extracts the Authorization header and validates it using the existing
    /// `ReportApiKey::validate_value()` function. Preserves all existing
    /// security checks and validation logic.
    #[instrument(err, skip_all)]
    async fn authenticate(&self, headers: &HeaderMap) -> anyhow::Result<AuthContext> {
        // Extract Authorization header
        let header_value = headers
            .get(AUTHORIZATION)
            .context("Missing Authorization header")?;

        let header_str = header_value
            .to_str()
            .context("Failed to parse Authorization header")?;

        // Call existing validation logic (protobuf decode, KMS decrypt, nonce validation, etc.)
        let (account_id, key_id) = ReportApiKey::validate_value(header_str)
            .await
            .context("Failed to validate API key")?;

        Ok(AuthContext { account_id, key_id })
    }
}

/// Test authentication implementation that returns pre-configured claims
///
/// This implementation bypasses all validation and returns a fixed `AuthContext`
/// regardless of the request contents. This enables integration tests to run
/// without requiring real JWT tokens or API keys.
///
/// # Usage in Tests
/// ```rust,ignore
/// let auth_provider = Arc::new(FixedAuthProvider::new("test_account_123", 99999));
/// let state = AppState { auth_provider, /* ... */ };
/// let router = create_router_with_state(state);
/// // Now all requests will be authenticated as test_account_123
/// ```
#[cfg(any(test, feature = "test-support"))]
#[derive(Clone)]
pub struct FixedAuthProvider {
    context: AuthContext,
}

#[cfg(any(test, feature = "test-support"))]
impl FixedAuthProvider {
    /// Create a new FixedAuthProvider with pre-configured authentication context
    ///
    /// # Parameters
    /// - `account_id`: The account ID to return for all authentication requests
    /// - `key_id`: The key ID to return for all authentication requests
    pub fn new(account_id: impl Into<String>, key_id: u32) -> Self {
        Self {
            context: AuthContext {
                account_id: account_id.into(),
                key_id,
            },
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
#[async_trait]
impl AuthProvider for FixedAuthProvider {
    /// Returns the pre-configured authentication context without validation
    ///
    /// Ignores all request headers and returns the fixed context that was
    /// provided during construction. This enables tests to control exactly
    /// which account/key is authenticated.
    async fn authenticate(&self, _headers: &HeaderMap) -> anyhow::Result<AuthContext> {
        Ok(self.context.clone())
    }
}
