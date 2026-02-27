use std::sync::Arc;

use chrono::Utc;
use ulid::Ulid;

use super::error::IngestError;
use super::model::ImageStorage;
use super::model::PresignedIngestUrl;
use crate::shared::record::image_record::ImageContentType;
use crate::shared::record::image_record::ImageId;
use crate::shared::record::image_record::ImageRecord;
use crate::shared::record::image_record::ImageRecordRepository;
use crate::shared::record::image_record::ImageStatus;

pub struct CreateImageIngestPresignedUrlCommand {
    pub content_type: String,
    pub file_name: String,
}

pub struct CreateImageIngestPresignedUrlCommandExecutor {
    image_repository: Arc<dyn ImageRecordRepository>,
    image_storage: Arc<dyn ImageStorage>,
    bucket: String,
    expiry_secs: u64,
}

impl CreateImageIngestPresignedUrlCommandExecutor {
    pub fn new(
        image_repository: Arc<dyn ImageRecordRepository>,
        image_storage: Arc<dyn ImageStorage>,
        bucket: String,
        expiry_secs: u64,
    ) -> Self {
        Self {
            image_repository,
            image_storage,
            bucket,
            expiry_secs,
        }
    }

    pub async fn execute(
        &self,
        command: CreateImageIngestPresignedUrlCommand,
    ) -> Result<PresignedIngestUrl, IngestError> {
        let content_type_enum = ImageContentType::try_from(command.content_type.as_str())
            .map_err(IngestError::UnsupportedContentType)?;

        validate_file_name(&command.file_name)?;

        let image_id = ImageId::new(Ulid::new().to_string());
        let object_key = format!("ingest/{}", image_id.as_ref());

        let now = Utc::now();
        let record = ImageRecord {
            id: image_id.clone(),
            status: ImageStatus::Pending,
            content_type: content_type_enum,
            file_name: command.file_name,
            size_bytes: None,
            object_key: object_key.clone(),
            created_at: now,
            updated_at: now,
        };

        self.image_repository.save(record).await?;

        let presigned_upload = self
            .image_storage
            .generate_presigned_ingest_url(
                &self.bucket,
                &object_key,
                &command.content_type,
                self.expiry_secs,
            )
            .await?;

        Ok(PresignedIngestUrl {
            image_id,
            upload_url: presigned_upload.upload_url,
            expires_at: presigned_upload.expires_at,
        })
    }
}

fn validate_file_name(file_name: &str) -> Result<(), IngestError> {
    if file_name.is_empty() {
        return Err(IngestError::InvalidFileName(
            "file name must not be empty".to_string(),
        ));
    }
    if file_name.len() > 255 {
        return Err(IngestError::InvalidFileName(
            "file name must not exceed 255 characters".to_string(),
        ));
    }
    if file_name.chars().any(|c| (c as u32) <= 31) {
        return Err(IngestError::InvalidFileName(
            "file name must not contain control characters".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::feature::image::ingest::error::StorageError;
    use crate::feature::image::ingest::model::ObjectMetadata;
    use crate::feature::image::ingest::model::PresignedUploadUrl;
    use crate::shared::record::image_record::RepositoryError;

    struct StubImageRecordRepository;

    #[async_trait]
    impl ImageRecordRepository for StubImageRecordRepository {
        async fn save(&self, _image: ImageRecord) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn find_by_id(&self, _id: &ImageId) -> Result<Option<ImageRecord>, RepositoryError> {
            Ok(None)
        }

        async fn find_by_ids(&self, _ids: &[ImageId]) -> Result<Vec<ImageRecord>, RepositoryError> {
            Ok(vec![])
        }

        async fn update(
            &self,
            _id: &ImageId,
            _modifier: Box<dyn FnOnce(ImageRecord) -> ImageRecord + Send>,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    struct StubImageStorage;

    #[async_trait]
    impl ImageStorage for StubImageStorage {
        async fn generate_presigned_ingest_url(
            &self,
            _bucket: &str,
            object_key: &str,
            _content_type: &str,
            _expiry_secs: u64,
        ) -> Result<PresignedUploadUrl, StorageError> {
            Ok(PresignedUploadUrl {
                upload_url: format!("https://s3.example.com/{object_key}"),
                expires_at: Utc::now(),
            })
        }

        async fn get_object_metadata(
            &self,
            _bucket: &str,
            _object_key: &str,
        ) -> Result<ObjectMetadata, StorageError> {
            Ok(ObjectMetadata { size_bytes: 1024 })
        }

        async fn delete_object(
            &self,
            _bucket: &str,
            _object_key: &str,
        ) -> Result<(), StorageError> {
            Ok(())
        }
    }

    fn arrange_executor() -> CreateImageIngestPresignedUrlCommandExecutor {
        CreateImageIngestPresignedUrlCommandExecutor::new(
            Arc::new(StubImageRecordRepository),
            Arc::new(StubImageStorage),
            "test-bucket".to_string(),
            300,
        )
    }

    fn arrange_command() -> CreateImageIngestPresignedUrlCommand {
        CreateImageIngestPresignedUrlCommand {
            content_type: "image/jpeg".to_string(),
            file_name: "photo.jpg".to_string(),
        }
    }

    #[tokio::test]
    async fn sut_returns_presigned_url_when_command_is_valid() {
        // Arrange
        let sut = arrange_executor();
        let command = arrange_command();

        // Act
        let result = sut.execute(command).await;

        // Assert
        assert!(result.is_ok());
        let url = result.unwrap();
        assert!(!url.upload_url.is_empty());
        assert!(!url.image_id.as_ref().is_empty());
    }

    #[tokio::test]
    async fn sut_returns_error_when_content_type_is_not_in_allowlist() {
        // Arrange
        let sut = arrange_executor();
        let mut command = arrange_command();
        command.content_type = "image/bmp".to_string();

        // Act
        let result = sut.execute(command).await;

        // Assert
        assert!(matches!(
            result,
            Err(IngestError::UnsupportedContentType(_))
        ));
    }

    #[tokio::test]
    async fn sut_returns_error_when_content_type_is_wildcard() {
        // Arrange
        let sut = arrange_executor();
        let mut command = arrange_command();
        command.content_type = "image/*".to_string();

        // Act
        let result = sut.execute(command).await;

        // Assert
        assert!(matches!(
            result,
            Err(IngestError::UnsupportedContentType(_))
        ));
    }

    #[tokio::test]
    async fn sut_returns_error_when_content_type_is_parameterized() {
        // Arrange
        let sut = arrange_executor();
        let mut command = arrange_command();
        command.content_type = "image/jpeg;charset=utf-8".to_string();

        // Act
        let result = sut.execute(command).await;

        // Assert
        assert!(matches!(
            result,
            Err(IngestError::UnsupportedContentType(_))
        ));
    }

    #[tokio::test]
    async fn sut_returns_error_when_file_name_is_empty() {
        // Arrange
        let sut = arrange_executor();
        let mut command = arrange_command();
        command.file_name = String::new();

        // Act
        let result = sut.execute(command).await;

        // Assert
        assert!(matches!(result, Err(IngestError::InvalidFileName(_))));
    }

    #[tokio::test]
    async fn sut_returns_error_when_file_name_exceeds_max_length() {
        // Arrange
        let sut = arrange_executor();
        let mut command = arrange_command();
        // 256 characters exceeds the 255 character limit
        command.file_name = "a".repeat(256);

        // Act
        let result = sut.execute(command).await;

        // Assert
        assert!(matches!(result, Err(IngestError::InvalidFileName(_))));
    }

    #[tokio::test]
    async fn sut_returns_error_when_file_name_contains_control_character() {
        // Arrange
        let sut = arrange_executor();
        let mut command = arrange_command();
        // ASCII 0x01 is a control character (falls in range 0-31)
        command.file_name = "photo\x01.jpg".to_string();

        // Act
        let result = sut.execute(command).await;

        // Assert
        assert!(matches!(result, Err(IngestError::InvalidFileName(_))));
    }

    #[tokio::test]
    async fn sut_accepts_gif_content_type() {
        // Arrange
        let sut = arrange_executor();
        let mut command = arrange_command();
        command.content_type = "image/gif".to_string();

        // Act
        let result = sut.execute(command).await;

        // Assert
        assert!(result.is_ok());
    }
}
