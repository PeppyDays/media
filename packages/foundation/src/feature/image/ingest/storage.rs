use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use tracing::info;

use super::error::StorageError;
use super::model::ImageStorage;
use super::model::ObjectMetadata;
use super::model::PresignedUploadUrl;

pub struct S3ImageStorage {
    client: aws_sdk_s3::Client,
}

impl S3ImageStorage {
    pub fn new(client: aws_sdk_s3::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ImageStorage for S3ImageStorage {
    async fn generate_presigned_ingest_url(
        &self,
        bucket: &str,
        object_key: &str,
        content_type: &str,
        expiry_secs: u64,
    ) -> Result<PresignedUploadUrl, StorageError> {
        let presigning_config = aws_sdk_s3::presigning::PresigningConfig::builder()
            .expires_in(Duration::from_secs(expiry_secs))
            .build()
            .map_err(|e| StorageError::PresignFailed(e.to_string()))?;

        let presigned = self
            .client
            .put_object()
            .bucket(bucket)
            .key(object_key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| StorageError::PresignFailed(e.to_string()))?;

        // The AWS SDK Rust includes content-type in X-Amz-SignedHeaders when .content_type() is
        // set on a PUT presigned request, so S3 will reject uploads with a mismatched Content-Type.
        let signed_headers: Vec<_> = presigned.headers().collect();
        info!(
            signed_headers = ?signed_headers,
            "presigned PUT URL generated; content-type is enforced via signed headers"
        );

        let upload_url = presigned.uri().to_string();
        let expires_at = Utc::now() + chrono::Duration::seconds(expiry_secs as i64);

        Ok(PresignedUploadUrl {
            upload_url,
            expires_at,
        })
    }

    async fn get_object_metadata(
        &self,
        bucket: &str,
        object_key: &str,
    ) -> Result<ObjectMetadata, StorageError> {
        let response = self
            .client
            .head_object()
            .bucket(bucket)
            .key(object_key)
            .send()
            .await
            .map_err(|e| StorageError::MetadataFailed(e.to_string()))?;

        let size_bytes = response
            .content_length()
            .ok_or_else(|| StorageError::MetadataFailed("missing content-length".to_string()))?;

        Ok(ObjectMetadata { size_bytes })
    }

    async fn delete_object(&self, bucket: &str, object_key: &str) -> Result<(), StorageError> {
        self.client
            .delete_object()
            .bucket(bucket)
            .key(object_key)
            .send()
            .await
            .map_err(|e| StorageError::DeleteFailed(e.to_string()))?;

        Ok(())
    }
}
