## Purpose
This repo contains Archodex-private code specific to the archodex.com hosting environment. The code is fetched from GitHub in the archodex-backend build.rs script when its `archodex-com` feature is enabled.

## Functionality
- New customer account creation:
  - Picking a customer data AWS account to use
  - Creating a DynamoDB table for surrealdb inside it
- Fetching the API key secret for signing customer API keys
  - Retrieving the encrypted API key secret from SSM Parameter Store
  - Using an AWS KMS CMK to decrypt the API key secret
 
## Debugging
General Archodex backend debugging information should be documented in the public [Archodex/archodex-backend](https://github.com/Archodex/archodex-backend) repo. The following debugging information is relevant to archodex.com backends.

> [!WARNING]
> Confidential and PII information should only ever be logged at the `trace` level. This is true for all Archodex projects. Adhering to this rule ensures safety of requesting and sharing information at `debug` and higher levels.

### Useful Rust Tracing Filters
By default we log all `info` and higher (i.e. `warn` and `error`) levels. Log levels can be manipulated using the standard `RUST_LOG` environment variable according to the tracing_subscriber [EnvFilter directives](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives).

> [!IMPORTANT]
> If you set the `RUST_LOG` environment variable to a valid set of filters the default filter for all `info` level logs will no longer be honored. You likely want to keep the `info` level logs, which can be achieved by including `info` in the comma-separated set of filters. The examples below all keep `info` as the last filter for this reason.

The following are commonly useful log filters:

#### SurrealDB DynamoDB KVS Operations
**Filter:** `surrealdb_core::kvs::dynamodb=trace,surrealdb::core::kvs::tr=trace,info`

This is useful for debugging deep DynamoDB KVS engine implementation issues. It will show log entries as SurrealDB processes transactions and as the DynamoDB KVS engine executes reads and writes to DDB tables.
