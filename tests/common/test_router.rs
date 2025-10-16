// Test router helpers for integration testing

use axum::{Router, routing::get};

/// Creates a simple test router without authentication middleware
///
/// This router is useful for testing endpoints that don't require authentication,
/// or for testing handler logic in isolation from auth concerns.
///
/// # Examples
///
/// ```ignore
/// use tower::ServiceExt; //  for oneshot()
///
/// #[tokio::test]
/// async fn test_health_endpoint() {
///     let app = create_test_router();
///
///     let response = app
///         .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
///         .await
///         .unwrap();
///
///     assert_eq!(response.status(), StatusCode::OK);
/// }
/// ```
pub fn create_test_router() -> Router {
    Router::new()
        .route("/health", get(|| async { "Ok" }))
}
