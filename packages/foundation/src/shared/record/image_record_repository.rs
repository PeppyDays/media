use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use sqlx::FromRow;
use sqlx::Pool;
use sqlx::Postgres;

use super::image_record::ImageContentType;
use super::image_record::ImageId;
use super::image_record::ImageRecord;
use super::image_record::ImageRecordRepository;
use super::image_record::ImageStatus;
use super::image_record::RepositoryError;

pub struct PostgresImageRecordRepository {
    pool: Pool<Postgres>,
}

impl PostgresImageRecordRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct ImageRecordDataModel {
    id: String,
    status: String,
    content_type: String,
    file_name: String,
    size_bytes: Option<i64>,
    object_key: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ImageRecord> for ImageRecordDataModel {
    fn from(value: ImageRecord) -> Self {
        Self {
            id: value.id.into(),
            status: value.status.as_ref().to_string(),
            content_type: value.content_type.as_ref().to_string(),
            file_name: value.file_name,
            size_bytes: value.size_bytes,
            object_key: value.object_key,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl TryFrom<ImageRecordDataModel> for ImageRecord {
    type Error = RepositoryError;

    fn try_from(value: ImageRecordDataModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ImageId::new(value.id),
            status: ImageStatus::try_from(value.status.as_str())
                .map_err(RepositoryError::DataMapping)?,
            content_type: ImageContentType::try_from(value.content_type.as_str())
                .map_err(RepositoryError::DataMapping)?,
            file_name: value.file_name,
            size_bytes: value.size_bytes,
            object_key: value.object_key,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[async_trait]
impl ImageRecordRepository for PostgresImageRecordRepository {
    async fn save(&self, image: ImageRecord) -> Result<(), RepositoryError> {
        let data_model = ImageRecordDataModel::from(image);
        sqlx::query(
            "INSERT INTO image_records (id, status, content_type, file_name, size_bytes, object_key, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (id) DO UPDATE SET
                 status = EXCLUDED.status,
                 content_type = EXCLUDED.content_type,
                 file_name = EXCLUDED.file_name,
                 size_bytes = EXCLUDED.size_bytes,
                 object_key = EXCLUDED.object_key,
                 updated_at = EXCLUDED.updated_at",
        )
        .bind(&data_model.id)
        .bind(&data_model.status)
        .bind(&data_model.content_type)
        .bind(&data_model.file_name)
        .bind(data_model.size_bytes)
        .bind(&data_model.object_key)
        .bind(data_model.created_at)
        .bind(data_model.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: &ImageId) -> Result<Option<ImageRecord>, RepositoryError> {
        let data_model: Option<ImageRecordDataModel> = sqlx::query_as(
            "SELECT id, status, content_type, file_name, size_bytes, object_key, created_at, updated_at
             FROM image_records WHERE id = $1",
        )
        .bind(id.as_ref())
        .fetch_optional(&self.pool)
        .await?;

        data_model.map(ImageRecord::try_from).transpose()
    }

    async fn find_by_ids(&self, ids: &[ImageId]) -> Result<Vec<ImageRecord>, RepositoryError> {
        let id_strings: Vec<&str> = ids.iter().map(|id| id.as_ref()).collect();
        let data_models: Vec<ImageRecordDataModel> = sqlx::query_as(
            "SELECT id, status, content_type, file_name, size_bytes, object_key, created_at, updated_at
             FROM image_records WHERE id = ANY($1)",
        )
        .bind(&id_strings)
        .fetch_all(&self.pool)
        .await?;

        data_models.into_iter().map(ImageRecord::try_from).collect()
    }

    async fn update(
        &self,
        id: &ImageId,
        modifier: impl FnOnce(ImageRecord) -> ImageRecord + Send,
    ) -> Result<(), RepositoryError> {
        let Some(image) = self.find_by_id(id).await? else {
            return Ok(());
        };

        self.save(modifier(image)).await
    }
}
