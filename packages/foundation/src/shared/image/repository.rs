use std::str::FromStr;

use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use sqlx::FromRow;
use sqlx::PgPool;

use super::model::Image;
use super::model::ImageId;
use super::model::ImageRepository;
use super::model::ImageStatus;
use super::model::RepositoryError;

pub struct PostgresImageRepository {
    pool: PgPool,
}

impl PostgresImageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct ImageRow {
    id: String,
    status: String,
    content_type: String,
    file_name: String,
    size_bytes: Option<i64>,
    object_key: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ImageRow {
    fn into_domain(self) -> Image {
        Image {
            id: ImageId(self.id),
            status: ImageStatus::from_str(&self.status)
                .unwrap_or_else(|_| panic!("unknown image status in database: {}", self.status)),
            content_type: self.content_type,
            file_name: self.file_name,
            size_bytes: self.size_bytes,
            object_key: self.object_key,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[async_trait]
impl ImageRepository for PostgresImageRepository {
    async fn save(&self, image: &Image) -> Result<(), RepositoryError> {
        sqlx::query(
            "INSERT INTO images (id, status, content_type, file_name, size_bytes, object_key, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&image.id.0)
        .bind(image.status.as_str())
        .bind(&image.content_type)
        .bind(&image.file_name)
        .bind(image.size_bytes)
        .bind(&image.object_key)
        .bind(image.created_at)
        .bind(image.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &ImageId) -> Result<Option<Image>, RepositoryError> {
        let row: Option<ImageRow> = sqlx::query_as(
            "SELECT id, status, content_type, file_name, size_bytes, object_key, created_at, updated_at
             FROM images WHERE id = $1",
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(ImageRow::into_domain))
    }

    async fn find_by_ids(&self, ids: &[ImageId]) -> Result<Vec<Image>, RepositoryError> {
        let id_strings: Vec<&str> = ids.iter().map(|id| id.0.as_str()).collect();
        let rows: Vec<ImageRow> = sqlx::query_as(
            "SELECT id, status, content_type, file_name, size_bytes, object_key, created_at, updated_at
             FROM images WHERE id = ANY($1)",
        )
        .bind(&id_strings)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(ImageRow::into_domain).collect())
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
        .bind(status.as_str())
        .bind(size_bytes)
        .bind(&id.0)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
