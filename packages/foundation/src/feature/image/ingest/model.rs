use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;

use super::error::StorageError;
use crate::shared::record::image_record::ImageId;

pub struct PresignedIngestUrl {
    pub image_id: ImageId,
    pub upload_url: String,
    pub expires_at: DateTime<Utc>,
}

pub struct PresignedUploadUrl {
    pub upload_url: String,
    pub expires_at: DateTime<Utc>,
}

pub struct ObjectMetadata {
    pub size_bytes: i64,
}

#[async_trait]
pub trait ImageStorage: Send + Sync {
    async fn generate_presigned_ingest_url(
        &self,
        bucket: &str,
        object_key: &str,
        content_type: &str,
        expiry_secs: u64,
    ) -> Result<PresignedUploadUrl, StorageError>;

    async fn get_object_metadata(
        &self,
        bucket: &str,
        object_key: &str,
    ) -> Result<ObjectMetadata, StorageError>;

    async fn delete_object(&self, bucket: &str, object_key: &str) -> Result<(), StorageError>;
}
