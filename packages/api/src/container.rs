use std::sync::Arc;
use std::time::Duration;

use foundation::config::Config;
use foundation::feature::image::ingest::command::CreateImageIngestPresignedUrlCommandExecutor;
use foundation::feature::image::ingest::storage::S3ImageStorage;
use foundation::shared::database::create_pool;
use foundation::shared::record::image_record::PostgresImageRecordRepository;
use foundation::shared::storage::create_s3_client;

#[derive(Clone)]
pub struct AppState {
    pub create_image_ingest_presigned_url: Arc<CreateImageIngestPresignedUrlCommandExecutor>,
}

impl AppState {
    pub async fn build(config: &Config) -> Self {
        let pool = create_pool(
            &config.database.dsn(),
            config.database.max_connections,
            Duration::from_secs(config.database.connection_timeout_secs),
            Duration::from_secs(config.database.idle_timeout_secs),
        )
        .await;

        let s3_client = create_s3_client(config.aws.region.clone()).await;

        let image_repository = Arc::new(PostgresImageRecordRepository::new(pool));
        let image_storage = Arc::new(S3ImageStorage::new(s3_client));

        let create_image_ingest_presigned_url =
            Arc::new(CreateImageIngestPresignedUrlCommandExecutor::new(
                image_repository,
                image_storage,
                config.storage.bucket_name.clone(),
                config.storage.upload_url_expiry_secs,
            ));

        Self {
            create_image_ingest_presigned_url,
        }
    }
}
