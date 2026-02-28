use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use foundation::feature::image::ingest::command::CreateImageIngestPresignedUrlCommand;
use foundation::feature::image::ingest::error::IngestError;
use serde::Deserialize;
use serde::Serialize;

use crate::container::AppState;

#[derive(Deserialize)]
pub struct CreatePresignedUrlRequest {
    pub content_type: String,
    pub file_name: String,
}

#[derive(Serialize)]
pub struct CreatePresignedUrlResponse {
    pub image_id: String,
    pub upload_url: String,
    pub expires_at: String,
}

pub async fn create_presigned_url(
    State(state): State<AppState>,
    Json(payload): Json<CreatePresignedUrlRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let command = CreateImageIngestPresignedUrlCommand {
        content_type: payload.content_type,
        file_name: payload.file_name,
    };

    let result = state
        .create_image_ingest_presigned_url
        .execute(command)
        .await?;

    let response = CreatePresignedUrlResponse {
        image_id: result.image_id.into(),
        upload_url: result.upload_url,
        expires_at: result.expires_at.to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub struct ApiError(IngestError);

impl From<IngestError> for ApiError {
    fn from(e: IngestError) -> Self {
        Self(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self.0 {
            IngestError::UnsupportedContentType(ct) => (
                StatusCode::BAD_REQUEST,
                format!("unsupported content type: {ct}"),
            ),
            IngestError::InvalidFileName(msg) => {
                (StatusCode::BAD_REQUEST, format!("invalid file name: {msg}"))
            }
            IngestError::Repository(e) => {
                tracing::error!("repository error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_string(),
                )
            }
            IngestError::Storage(e) => {
                tracing::error!("storage error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_string(),
                )
            }
        };

        (status, Json(serde_json::json!({ "message": message }))).into_response()
    }
}
