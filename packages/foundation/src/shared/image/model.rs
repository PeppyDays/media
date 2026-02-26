use std::fmt;

use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;

#[derive(Debug)]
pub struct Image {
    pub id: ImageId,
    pub status: ImageStatus,
    pub content_type: ImageContentType,
    pub file_name: String,
    pub size_bytes: Option<i64>,
    pub object_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ImageId(String);

impl ImageId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

impl From<ImageId> for String {
    fn from(id: ImageId) -> Self {
        id.0
    }
}

impl fmt::Display for ImageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for ImageId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug)]
pub enum ImageStatus {
    Pending,
    Ready,
    Failed,
}

impl AsRef<str> for ImageStatus {
    fn as_ref(&self) -> &str {
        match self {
            Self::Pending => "Pending",
            Self::Ready => "Ready",
            Self::Failed => "Failed",
        }
    }
}

impl TryFrom<&str> for ImageStatus {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "Pending" => Ok(Self::Pending),
            "Ready" => Ok(Self::Ready),
            "Failed" => Ok(Self::Failed),
            _ => Err(format!("unknown image status: {s}")),
        }
    }
}

#[derive(Debug)]
pub enum ImageContentType {
    Jpeg,
    Png,
    WebP,
    Avif,
}

impl AsRef<str> for ImageContentType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::WebP => "image/webp",
            Self::Avif => "image/avif",
        }
    }
}

impl TryFrom<&str> for ImageContentType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "image/jpeg" => Ok(Self::Jpeg),
            "image/png" => Ok(Self::Png),
            "image/webp" => Ok(Self::WebP),
            "image/avif" => Ok(Self::Avif),
            _ => Err(format!("unknown image content type: {s}")),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("data mapping error: {0}")]
    DataMapping(String),
}

#[async_trait]
pub trait ImageRepository: Send + Sync {
    async fn save(&self, image: Image) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: &ImageId) -> Result<Option<Image>, RepositoryError>;
    async fn find_by_ids(&self, ids: &[ImageId]) -> Result<Vec<Image>, RepositoryError>;
    async fn update(
        &self,
        id: &ImageId,
        modifier: impl FnOnce(Image) -> Image + Send,
    ) -> Result<(), RepositoryError>;
}
