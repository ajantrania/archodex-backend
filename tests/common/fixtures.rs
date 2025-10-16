// Test data fixtures and factories

use serde::{Deserialize, Serialize};

/// Test account fixture
///
/// Simplified version of the production Account struct for testing purposes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestAccount {
    pub id: String,
    pub name: String,
    pub salt: Vec<u8>,
}

/// Creates a test account with the given ID and name
///
/// # Examples
///
/// ```ignore
/// let account = create_test_account("test_account_001", "Test Account");
/// ```
pub fn create_test_account(id: &str, name: &str) -> TestAccount {
    TestAccount {
        id: id.to_string(),
        name: name.to_string(),
        salt: rand::random::<[u8; 16]>().to_vec(),
    }
}

/// Creates a test authentication token for the given account ID
///
/// This generates a token in the format "test_token_{account_id}" which
/// is recognized by the #[cfg(test)] bypass logic in ReportApiKey::validate_value().
///
/// # Examples
///
/// ```ignore
/// let token = create_test_auth_token("test_account_123");
/// // Returns: "test_token_test_account_123"
/// ```
pub fn create_test_auth_token(account_id: &str) -> String {
    format!("test_token_{}", account_id)
}

/// Creates a minimal but valid report request for testing report ingestion logic
///
/// The simplified payload includes:
/// - 2 resource captures (1 flat API resource, 1 nested Kubernetes structure)
/// - 2 event captures (1 API access event, 1 K8s service targeting event)
///
/// This tests the core business logic:
/// - Resource tree upsert with nested contains relationships
/// - Principal chain creation and linking
/// - Event relationship creation between principals and resources
/// - Timestamp handling (first_seen_at, last_seen_at)
pub fn create_test_report_request() -> serde_json::Value {
    serde_json::json!({
        "resource_captures": [
            {
                "type": "Test API",
                "id": "api.example.com",
                "globally_unique": null,
                "first_seen_at": "2025-10-16T01:00:00Z",
                "last_seen_at": "2025-10-16T02:00:00Z",
                "attributes": null,
                "contains": null
            },
            {
                "type": "Kubernetes Cluster",
                "id": "test-cluster-123",
                "globally_unique": null,
                "first_seen_at": "2025-10-16T01:00:00Z",
                "last_seen_at": "2025-10-16T02:00:00Z",
                "attributes": null,
                "contains": [
                    {
                        "type": "Namespace",
                        "id": "prod",
                        "globally_unique": null,
                        "first_seen_at": "2025-10-16T01:00:00Z",
                        "last_seen_at": "2025-10-16T02:00:00Z",
                        "attributes": null,
                        "contains": [
                            {
                                "type": "Service",
                                "id": "api-service",
                                "globally_unique": null,
                                "first_seen_at": "2025-10-16T01:00:00Z",
                                "last_seen_at": "2025-10-16T02:00:00Z",
                                "attributes": null,
                                "contains": null
                            }
                        ]
                    }
                ]
            }
        ],
        "event_captures": [
            {
                "principals": [
                    {
                        "id": [
                            ["Kubernetes Cluster", "test-cluster-123"],
                            ["Namespace", "prod"],
                            ["Container", "test-container"]
                        ],
                        "event": null
                    }
                ],
                "resources": [
                    [
                        ["Test API", "api.example.com"]
                    ]
                ],
                "events": [
                    {
                        "type": "Accessed",
                        "first_seen_at": "2025-10-16T01:30:00Z",
                        "last_seen_at": "2025-10-16T01:45:00Z"
                    }
                ]
            },
            {
                "principals": [
                    {
                        "id": [
                            ["Kubernetes Cluster", "test-cluster-123"],
                            ["Namespace", "prod"],
                            ["Service", "frontend"]
                        ],
                        "event": null
                    }
                ],
                "resources": [
                    [
                        ["Kubernetes Cluster", "test-cluster-123"],
                        ["Namespace", "prod"],
                        ["Service", "api-service"]
                    ]
                ],
                "events": [
                    {
                        "type": "Targeted",
                        "first_seen_at": "2025-10-16T01:00:00Z",
                        "last_seen_at": "2025-10-16T02:00:00Z"
                    }
                ]
            }
        ]
    })
}

/// Test user fixture
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestUser {
    pub id: String,
}

/// Creates a test user with the given ID
pub fn create_test_user(id: &str) -> TestUser {
    TestUser {
        id: id.to_string(),
    }
}

/// Generates a random account salt for testing
pub fn create_test_account_salt() -> Vec<u8> {
    rand::random::<[u8; 16]>().to_vec()
}
