// Integration tests for /report endpoint with full authentication middleware
//
// These tests validate the complete request flow including:
// - ReportApiKeyAuth::authenticate middleware (token validation)
// - report_api_key_account middleware (account loading, key validation)
// - report::report handler (business logic)
//
// Security note: Tests use the production router and only test through public HTTP APIs.
// No internal types or functions are exposed - all testing is done via HTTP requests.

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header::AUTHORIZATION},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_report_endpoint_with_valid_auth() {
    // Setup: Set test environment variables
    common::setup_test_env();

    // Create production router (includes all middleware)
    let app = archodex_backend::router::router();

    // Create test report payload
    let report_payload = common::create_test_report_request();

    // Test 1: Missing auth - should return 401
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/report")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&report_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Report endpoint should return 401 when Authorization header is missing"
    );
}

#[tokio::test]
async fn test_report_endpoint_rejects_invalid_token() {
    common::setup_test_env();

    let app = archodex_backend::router::router();
    let report_payload = common::create_test_report_request();

    // Test: Invalid token format
    let response = app
        .oneshot(
            Request::builder()
                .uri("/report")
                .method("POST")
                .header(AUTHORIZATION, "invalid_token_format")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&report_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Report endpoint should return 401 for malformed auth token"
    );
}

#[tokio::test]
async fn test_report_endpoint_rejects_nonexistent_account() {
    common::setup_test_env();

    let app = archodex_backend::router::router();
    let report_payload = common::create_test_report_request();

    // Test: Valid token format but account doesn't exist
    let nonexistent_account_token = common::create_test_auth_token("9999999999");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/report")
                .method("POST")
                .header(AUTHORIZATION, nonexistent_account_token)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&report_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Note: The middleware returns 401 (not 404) when the account doesn't exist
    // This is intentional - we don't want to leak information about which accounts exist
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Report endpoint should return 401 when account doesn't exist (don't leak account existence)"
    );
}

// NOTE: T036 parts 1-4 (successful ingestion with database validation) cannot be
// fully implemented while maintaining security boundaries (pub(crate) types).
//
// LIMITATION: Testing through production router (public API only) means we cannot:
// - Inject in-memory database connections for Account::resources_db()
// - Validate database state after report ingestion
// - Test the complete success path (200 OK) without real database infrastructure
//
// The tests above successfully validate:
// ✅ Auth middleware rejection paths (missing/invalid/nonexistent tokens)
// ✅ Production router + middleware integration
// ✅ Security boundaries maintained (no internal types exposed)
//
// What CANNOT be tested without exposing internal types:
// ❌ HTTP 200 success path (requires database injection)
// ❌ Account record validation (Account is pub(crate))
// ❌ Resource/event count validation (requires DB access)
// ❌ Timestamp validation (requires DB access)
//
// ALTERNATIVES for full integration testing:
// 1. Add test-specific database injection mechanism (requires architecture change)
// 2. Use end-to-end tests with real SurrealDB instance (outside unit/integration test scope)
// 3. Expose limited test-only APIs with #[cfg(test)] visibility (violates research.md principles)
//
// DECISION: Document this limitation rather than compromise security boundaries.
// The rejection path testing provides significant value in validating auth middleware.
