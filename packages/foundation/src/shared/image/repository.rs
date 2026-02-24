use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use sqlx::FromRow;
use sqlx::Pool;
use sqlx::Postgres;

use super::model::Image;
use super::model::ImageContentType;
use super::model::ImageId;
use super::model::ImageRepository;
use super::model::ImageStatus;
use super::model::RepositoryError;

pub struct PostgresImageRepository {
    pool: Pool<Postgres>,
}

impl PostgresImageRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct ImageDataModel {
    id: String,
    status: String,
    content_type: String,
    file_name: String,
    size_bytes: Option<i64>,
    object_key: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Image> for ImageDataModel {
    fn from(value: Image) -> Self {
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

impl TryFrom<ImageDataModel> for Image {
    type Error = RepositoryError;

    fn try_from(value: ImageDataModel) -> Result<Self, Self::Error> {
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
impl ImageRepository for PostgresImageRepository {
    async fn save(&self, image: Image) -> Result<(), RepositoryError> {
        let data_model = ImageDataModel::from(image);
        sqlx::query(
            "INSERT INTO images (id, status, content_type, file_name, size_bytes, object_key, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
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

    async fn find_by_id(&self, id: &ImageId) -> Result<Option<Image>, RepositoryError> {
        let data_model: Option<ImageDataModel> = sqlx::query_as(
            "SELECT id, status, content_type, file_name, size_bytes, object_key, created_at, updated_at
             FROM images WHERE id = $1",
        )
        .bind(id.as_ref())
        .fetch_optional(&self.pool)
        .await?;

        data_model.map(Image::try_from).transpose()
    }

    async fn find_by_ids(&self, ids: &[ImageId]) -> Result<Vec<Image>, RepositoryError> {
        let id_strings: Vec<&str> = ids.iter().map(|id| id.as_ref()).collect();
        let data_models: Vec<ImageDataModel> = sqlx::query_as(
            "SELECT id, status, content_type, file_name, size_bytes, object_key, created_at, updated_at
             FROM images WHERE id = ANY($1)",
        )
        .bind(&id_strings)
        .fetch_all(&self.pool)
        .await?;

        data_models.into_iter().map(Image::try_from).collect()
    }

    async fn update_status(
        &self,
        id: &ImageId,
        status: &ImageStatus,
        size_bytes: Option<i64>,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            "UPDATE images SET status = $1, size_bytes = COALESCE($2, size_bytes), updated_at = now()
             WHERE id = $3",
        )
        .bind(status.as_ref())
        .bind(size_bytes)
        .bind(id.as_ref())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
