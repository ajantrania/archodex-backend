# archodex-backend Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-10

## Active Technologies
- Rust 2024 edition (workspace configured for edition 2024) + axum 0.7, surrealdb 2.3.7, aes-gcm 0.10.3, prost 0.13.5 (protobuf), tokio 1.47 (001-rate-limits-we)
- Rust 2024 edition (workspace configured for edition 2024) + axum 0.7.9, surrealdb 2.3.7, tokio 1.47.1, prost 0.13.5 (protobuf), aes-gcm 0.10.3 (002-specs-001-rate)
- SurrealDB 2.3.7 (Archodex fork with DynamoDB backend for managed service, RocksDB for self-hosted) (002-specs-001-rate)
- Rust 2024 edition + axum 0.7.9, surrealdb 2.3.7, tokio 1.47.1 (003-db-dependency-injection)
- SurrealDB (RocksDB for self-hosted, DynamoDB backend for archodex-com managed service) (003-db-dependency-injection)

## Project Structure
```
src/
tests/
```

## Commands
cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style
Rust 2024 edition (workspace configured for edition 2024): Follow standard conventions

## Recent Changes
- 003-db-dependency-injection: Added Rust 2024 edition + axum 0.7.9, surrealdb 2.3.7, tokio 1.47.1
- 002-specs-001-rate: Added Rust 2024 edition (workspace configured for edition 2024) + axum 0.7.9, surrealdb 2.3.7, tokio 1.47.1, prost 0.13.5 (protobuf), aes-gcm 0.10.3
- 001-rate-limits-we: Added Rust 2024 edition (workspace configured for edition 2024) + axum 0.7, surrealdb 2.3.7, aes-gcm 0.10.3, prost 0.13.5 (protobuf), tokio 1.47

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
