use std::{
    collections::HashMap,
    sync::LazyLock,
    time::{Duration, Instant},
};

use tokio::{
    sync::{OnceCell, RwLock},
    time::sleep,
};
use tracing::{Span, info, instrument, trace, warn};

use archodex_error::{
    anyhow::{self, Context as _, anyhow, bail, ensure},
    conflict,
};

const DYNAMODB_TABLE_PREFIX: &str = "archodex-service-data-";

#[must_use]
pub fn begin_readonly_statement() -> surrealdb::sql::statements::BeginStatement {
    #[cfg(feature = "archodex-com")]
    {
        let mut begin = surrealdb::sql::statements::BeginStatement::default();
        begin.readonly = true;
        begin
    }

    #[cfg(not(feature = "archodex-com"))]
    {
        surrealdb::sql::statements::BeginStatement::default()
    }
}

async fn aws_config() -> &'static aws_config::SdkConfig {
    static AWS_CONFIG: OnceCell<aws_config::SdkConfig> = OnceCell::const_new();

    AWS_CONFIG
        .get_or_init(|| async { aws_config::load_from_env().await })
        .await
}

#[instrument(err)]
async fn aws_identity()
-> anyhow::Result<&'static aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityOutput> {
    static AWS_IDENTITY: OnceCell<
        aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityOutput,
    > = OnceCell::const_new();

    AWS_IDENTITY
        .get_or_try_init(|| async {
            aws_sdk_sts::Client::new(aws_config().await)
                .get_caller_identity()
                .send()
                .await
                .context("Failed to get AWS identity")
        })
        .await
}

#[instrument(err)]
async fn aws_account_id() -> anyhow::Result<&'static String> {
    static AWS_ACCOUNT_ID: OnceCell<String> = OnceCell::const_new();

    AWS_ACCOUNT_ID
        .get_or_try_init(|| async {
            if let Ok(account_id) = std::env::var("AWS_ACCOUNT_ID") {
                Ok(account_id)
            } else {
                aws_identity()
                    .await?
                    .account()
                    .map(std::string::ToString::to_string)
                    .ok_or_else(|| anyhow!("AWS STS GetCallerIdentity response missing Account"))
            }
        })
        .await
}

#[instrument(err)]
async fn aws_partition() -> anyhow::Result<&'static String> {
    static AWS_PARTITION: OnceCell<String> = OnceCell::const_new();

    AWS_PARTITION
        .get_or_try_init(|| async {
            if let Ok(aws_partition) = std::env::var("AWS_PARTITION") {
                Ok(aws_partition)
            } else {
                let identity = aws_identity().await?;

                let arn = identity
                    .arn()
                    .ok_or_else(|| anyhow!("AWS STS GetCallerIdentity response missing Arn"))?;

                let Some(partition) = arn.split(':').nth(1) else {
                    bail!("Invalid AWS ARN in STS GetCallerIdentity response");
                };

                Ok(partition.to_string())
            }
        })
        .await
}

async fn aws_cloudwatch_client() -> &'static aws_sdk_cloudwatch::Client {
    static CLIENT: OnceCell<aws_sdk_cloudwatch::Client> = OnceCell::const_new();

    CLIENT
        .get_or_init(|| async { aws_sdk_cloudwatch::Client::new(aws_config().await) })
        .await
}

async fn aws_organizations_client() -> &'static aws_sdk_organizations::Client {
    static CLIENT: OnceCell<aws_sdk_organizations::Client> = OnceCell::const_new();

    CLIENT
        .get_or_init(|| async { aws_sdk_organizations::Client::new(aws_config().await) })
        .await
}

async fn aws_dynamodb_client() -> &'static aws_sdk_dynamodb::Client {
    static CLIENT: OnceCell<aws_sdk_dynamodb::Client> = OnceCell::const_new();

    CLIENT
        .get_or_init(|| async { aws_sdk_dynamodb::Client::new(aws_config().await) })
        .await
}

#[instrument(err)]
async fn aws_customer_data_account_role_arn(
    customer_data_aws_account_id: &str,
) -> anyhow::Result<String> {
    let aws_partition = aws_partition().await?;

    Ok(format!(
        "arn:{aws_partition}:iam::{customer_data_aws_account_id}:role/BackendAPICustomerDataManagementRole",
    ))
}

#[instrument(err)]
async fn aws_dynamodb_client_for_customer_data_account(
    archodex_account_id: &str,
    customer_data_aws_account_id: &str,
) -> anyhow::Result<aws_sdk_dynamodb::Client> {
    static CLIENTS: LazyLock<RwLock<HashMap<String, aws_sdk_dynamodb::Client>>> =
        LazyLock::new(|| RwLock::new(HashMap::new()));

    let clients_by_account_id = CLIENTS.read().await;

    if let Some(client) = clients_by_account_id.get(customer_data_aws_account_id) {
        Ok(client.clone())
    } else {
        drop(clients_by_account_id);

        let mut clients_by_account_id = CLIENTS.write().await;

        if let Some(client) = clients_by_account_id.get(customer_data_aws_account_id) {
            Ok(client.clone())
        } else {
            let customer_data_account_role_arn =
                aws_customer_data_account_role_arn(customer_data_aws_account_id).await?;

            let provider =
                aws_config::sts::AssumeRoleProvider::builder(customer_data_account_role_arn)
                    .session_name(format!(
                        "create-account-{archodex_account_id}-service-data-table"
                    ))
                    .build()
                    .await;

            let config = aws_config::from_env()
                .credentials_provider(provider)
                .load()
                .await;

            let client = aws_sdk_dynamodb::Client::new(&config);

            clients_by_account_id.insert(customer_data_aws_account_id.to_string(), client.clone());

            Ok(client)
        }
    }
}

#[instrument(err)]
async fn get_customer_data_aws_account_ids() -> anyhow::Result<Vec<String>> {
    let customer_data_ou_id =
        std::env::var("CUSTOMER_DATA_OU_ID").expect("Missing CUSTOMER_DATA_OU_ID env var");

    let client = aws_organizations_client().await;

    let account_list = client
        .list_accounts_for_parent()
        .parent_id(customer_data_ou_id)
        .send()
        .await
        .context("Failed to list customer data AWS accounts")?;

    account_list
        .accounts
        .ok_or_else(|| anyhow!("Response from AWS Organizations account list missing `Accounts`"))?
        .into_iter()
        .map(|account| {
            account
                .id
                .ok_or_else(|| anyhow!("Response from AWS Organizations account list missing `Id`"))
        })
        .collect::<anyhow::Result<_>>()
}

#[instrument(err)]
async fn select_customer_data_aws_account() -> anyhow::Result<String> {
    use aws_sdk_cloudwatch::types::{Dimension, Metric, MetricDataQuery, MetricStat, Statistic};

    let aws_account_ids = get_customer_data_aws_account_ids()
        .await
        .context("Failed to get customer data AWS account IDs")?;

    let client = aws_cloudwatch_client().await;

    let mut req = client
        .get_metric_data()
        .start_time((std::time::SystemTime::now() - std::time::Duration::from_secs(10 * 60)).into())
        .end_time(std::time::SystemTime::now().into());

    for account_id in aws_account_ids {
        req = req.metric_data_queries(
            MetricDataQuery::builder()
                .id(format!("table_count_{account_id}"))
                .account_id(account_id)
                .metric_stat(
                    MetricStat::builder()
                        .period(60)
                        .stat(Statistic::Maximum.to_string())
                        .metric(
                            Metric::builder()
                                .namespace("AWS/Usage")
                                .metric_name("ResourceCount")
                                .dimensions(
                                    Dimension::builder()
                                        .name("Service")
                                        .value("DynamoDB")
                                        .build(),
                                )
                                .dimensions(
                                    Dimension::builder().name("Type").value("Resource").build(),
                                )
                                .dimensions(
                                    Dimension::builder()
                                        .name("Resource")
                                        .value("TableCount")
                                        .build(),
                                )
                                .dimensions(
                                    Dimension::builder().name("Class").value("None").build(),
                                )
                                .build(),
                        )
                        .build(),
                )
                .build(),
        );
    }

    let metric_data = req
        .send()
        .await
        .context("Failed to get number of DynamoDB tables in customer data accounts")?;

    let metrics = metric_data.metric_data_results.ok_or_else(|| {
        anyhow!("Response from CloudWatch GetMetricData missing `MetricDataResults`")
    })?;

    let table_counts = metrics
        .into_iter()
        .map(|metric| {
            let id = metric
                .id
                .ok_or_else(|| anyhow!("Metric missing `Id`"))?
                .trim_start_matches("table_count_")
                .to_owned();

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let num_tables = *metric
                .values
                .ok_or_else(|| anyhow!("Metric missing `Values`"))?
                .first()
                .unwrap_or(&0f64) as u32;

            Ok((id, num_tables))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    info!("Customer data accounts table counts: {table_counts:#?}");

    let aws_account_id = table_counts
        .into_iter()
        .min_by_key(|table_count| table_count.1)
        .expect("No AWS customer data accounts?")
        .0;

    Ok(aws_account_id)
}

/// # Errors
///
/// Will return `Err` if the account service database cannot be created for any reason.
#[instrument(err)]
#[allow(clippy::too_many_lines)]
pub async fn create_account_service_database(archodex_account_id: &str) -> anyhow::Result<String> {
    use aws_sdk_dynamodb::{
        error::ProvideErrorMetadata,
        operation::{
            create_table::CreateTableError::ResourceInUseException,
            update_continuous_backups::UpdateContinuousBackupsError,
        },
        types::{
            AttributeDefinition, BillingMode, KeySchemaElement, KeyType,
            PointInTimeRecoverySpecification, ScalarAttributeType, SseSpecification, SseType,
            TableStatus,
        },
    };

    let aws_config = aws_config().await;

    let aws_partition = aws_partition().await?;
    let aws_region = aws_config
        .region()
        .context("Missing default region for AWS configuration profile")?;
    let backend_aws_account_id = aws_account_id().await?;

    let customer_data_aws_account_id = select_customer_data_aws_account()
        .await
        .with_context(|| "Failed to select customer data AWS account")?;

    let service_data_surrealdb_url = format!(
        "dynamodb://arn:{aws_partition}:dynamodb:{aws_region}:{customer_data_aws_account_id}:table/{DYNAMODB_TABLE_PREFIX}",
    );

    let client = aws_dynamodb_client_for_customer_data_account(
        archodex_account_id,
        &customer_data_aws_account_id,
    )
    .await?;

    let table_name = format!("{DYNAMODB_TABLE_PREFIX}a{archodex_account_id}-resources");

    info!("Creating DynamoDB table {table_name}...");

    let table_arn = match client
        .create_table()
        .table_name(&table_name)
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("pk")
                .attribute_type(ScalarAttributeType::B)
                .build()?,
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("sk")
                .attribute_type(ScalarAttributeType::B)
                .build()?,
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("pk")
                .key_type(KeyType::Hash)
                .build()?,
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("sk")
                .key_type(KeyType::Range)
                .build()?,
        )
        .billing_mode(BillingMode::PayPerRequest)
        .sse_specification(
            SseSpecification::builder()
                .enabled(true)
                .sse_type(SseType::Kms)
                .kms_master_key_id(format!("arn:aws:kms:{aws_region}:{backend_aws_account_id}:alias/ArchodexBackendCustomerDataKey"))
                .build(),
        )
        .send()
        .await
    {
        Ok(result) => result
            .table_description()
            .context("Table description missing from CreateTable response")?
            .table_arn()
            .context("Table ARN missing from CreateTable response table description")?
            .to_string(),
        Err(err) => match err.into_service_error() {
            ResourceInUseException(_) => conflict!("Account already exists"),
            err => bail!(err),
        },
    };

    info!("Table {table_name} created");

    info!("Waiting for table {table_name} to become available...");

    let start = Instant::now();

    loop {
        trace!("Describing table {table_name}...");

        let table_desc = client
            .describe_table()
            .table_name(&table_name)
            .send()
            .await?;

        let status = table_desc
            .table()
            .context("Table description missing from DescribeTable response")?
            .table_status()
            .context("Table status missing from DescribeTable response")?;

        trace!("Table {table_name} status is {status}");

        if status == &TableStatus::Active {
            break;
        }

        ensure!(
            Instant::now().duration_since(start) <= Duration::from_secs(30),
            "Table {table_name} failed to become available within 30 seconds"
        );

        sleep(Duration::from_secs(1)).await;
    }

    info!("Table {table_name} is available");

    info!("Adding Resource Policy to table {table_name}...");

    let policy = serde_json::to_string_pretty(&serde_json::json!({
        "Version": "2012-10-17",
        "Statement": [
            {
                "Effect": "Allow",
                "Principal": {
                    "AWS": format!("arn:{aws_partition}:iam::{backend_aws_account_id}:root")
                },
                "Action": [
                    "dynamodb:BatchGetItem",
                    "dynamodb:BatchWriteItem",
                    "dynamodb:ConditionCheckItem",
                    "dynamodb:DeleteItem",
                    "dynamodb:DeleteTable",
                    "dynamodb:DescribeTable",
                    "dynamodb:DescribeTimeToLive",
                    "dynamodb:GetItem",
                    "dynamodb:PutItem",
                    "dynamodb:Query",
                    "dynamodb:UpdateItem",
                    "dynamodb:UpdateTable",
                ],
                "Resource": "*",
                "Condition": {
                    "ArnLike": {
                        "aws:PrincipalArn": [
                            format!("arn:{aws_partition}:iam::{backend_aws_account_id}:role/ArchodexBackendAPIRole"),
                            format!("arn:{aws_partition}:iam::{backend_aws_account_id}:role/aws-reserved/sso.amazonaws.com/us-west-2/AWSReservedSSO_AdministratorAccess_*")
                        ]
                    }
                }
            }
        ]
    }))
    .with_context(|| format!("Failed to serialize Resource Policy for table {table_name}"))?;

    client
        .put_resource_policy()
        .resource_arn(table_arn)
        .policy(policy)
        .send()
        .await?;

    info!("Resource Policy added to table {table_name}");

    info!("Enabling Point In Time Recovery for table {table_name}...");

    loop {
        match client
            .update_continuous_backups()
            .table_name(&table_name)
            .point_in_time_recovery_specification(
                PointInTimeRecoverySpecification::builder()
                    .point_in_time_recovery_enabled(true)
                    .build()
                    .with_context(|| {
                        format!(
                            "Failed to build DynamoDB PITR specification for table {table_name}"
                        )
                    })?,
            )
            .send()
            .await
        {
            Ok(_) => break,
            Err(err) => match err.into_service_error() {
                UpdateContinuousBackupsError::ContinuousBackupsUnavailableException(_) => (),
                err if err.code() == Some("UnknownOperationException") => {
                    warn!(
                        "Ignoring DynamoDB Point In Time Recovery unknown operation error, which is expected with DynamoDB Local"
                    );
                    break;
                }
                err => bail!("Failed to enable DynamoDB PITR for table {table_name}: {err:#?}"),
            },
        }

        trace!(
            "Table {table_name} is still enabling continuous backups, will retry enabling PITR..."
        );

        ensure!(
            Instant::now().duration_since(start) <= Duration::from_secs(30),
            "Table {table_name} failed to become available with PITR within 30 seconds"
        );

        sleep(Duration::from_secs(1)).await;
    }

    info!("Point In Time Recovery enabled for table {table_name}");

    Ok(service_data_surrealdb_url)
}

/// # Errors
///
/// Will return `Err` if the surrealdb URL is not a dynamodb Table or the delete table action errors
#[instrument(err, fields(table_arn=tracing::field::Empty))]
pub async fn delete_account_service_database(
    service_data_surrealdb_url: &str,
    account_id: &str,
) -> anyhow::Result<()> {
    let Some(table_arn_prefix) = service_data_surrealdb_url.strip_prefix("dynamodb://") else {
        bail!("Invalid service data SurrealDB URL")
    };

    let table_arn = format!("{table_arn_prefix}a{account_id}-resources");

    Span::current().record("table_arn", &table_arn);

    let client = aws_dynamodb_client().await;

    client
        .delete_table()
        .table_name(table_arn)
        .send()
        .await
        .context("Failed to delete account service database")?;

    Ok(())
}

/// # Panics
///
/// This function will panic if the encrypted API private key cannot be retrieved from AWS SSM Parameter Store, decoded
/// from base64, or decrypted by AWS KMS.
#[instrument]
pub async fn api_private_key() -> &'static aes_gcm::Key<aes_gcm::Aes128Gcm> {
    use base64::prelude::*;

    static DATA_KEY: OnceCell<aes_gcm::Key<aes_gcm::Aes128Gcm>> = OnceCell::const_new();

    DATA_KEY
        .get_or_init(|| async {
            let ssm_client = aws_sdk_ssm::Client::new(aws_config().await);

            let encrypted_data_key_base64 = ssm_client
                .get_parameter()
                .name("api_key_customer_data_key")
                .send()
                .await
                .expect("Failed to get API key")
                .parameter
                .expect("SSM GetParameter response missing Parameter")
                .value
                .expect("SSM GetParameter response missing Parameter value");

            let encrypted_data_key = BASE64_STANDARD
                .decode(encrypted_data_key_base64)
                .expect("Failed to decode API Keys encrypted data key");

            let encrypted_data_key = aws_smithy_types::Blob::new(encrypted_data_key);

            let kms_client = aws_sdk_kms::Client::new(aws_config().await);

            let data_key = kms_client
                .decrypt()
                .ciphertext_blob(encrypted_data_key)
                .encryption_context("Purpose", "APIKeys")
                .send()
                .await
                .expect("Failed to decrypt data key")
                .plaintext
                .expect("KMS Decrypt response missing Plaintext");

            aes_gcm::Key::<aes_gcm::Aes128Gcm>::clone_from_slice(data_key.into_inner().as_slice())
        })
        .await
}
