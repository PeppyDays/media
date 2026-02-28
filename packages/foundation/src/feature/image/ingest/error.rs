use crate::shared::record::image_record::RepositoryError;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("unsupported content type: {0}")]
    UnsupportedContentType(String),

    #[error("invalid file name: {0}")]
    InvalidFileName(String),

    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("presign failed: {0}")]
    PresignFailed(String),

    #[error("metadata failed: {0}")]
    MetadataFailed(String),

    #[error("delete failed: {0}")]
    DeleteFailed(String),
}
