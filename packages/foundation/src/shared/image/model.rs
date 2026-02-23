use std::fmt;
use std::str::FromStr;

use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;

#[derive(Debug)]
pub struct ImageId(pub String);

impl fmt::Display for ImageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug)]
pub enum ImageStatus {
    Pending,
    Ready,
    Failed,
}

impl ImageStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

impl FromStr for ImageStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "ready" => Ok(Self::Ready),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("unknown image status: {s}")),
        }
    }
}

#[derive(Debug)]
pub struct Image {
    pub id: ImageId,
    pub status: ImageStatus,
    pub content_type: String,
    pub file_name: String,
    pub size_bytes: Option<i64>,
    pub object_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[async_trait]
pub trait ImageRepository: Send + Sync {
    async fn save(&self, image: &Image) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: &ImageId) -> Result<Option<Image>, RepositoryError>;
    async fn find_by_ids(&self, ids: &[ImageId]) -> Result<Vec<Image>, RepositoryError>;
    async fn update_status(
        &self,
        id: &ImageId,
        status: &ImageStatus,
        size_bytes: Option<i64>,
    ) -> Result<(), RepositoryError>;
}
