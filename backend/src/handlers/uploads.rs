use std::path::PathBuf;

use axum::{extract::Multipart, http::StatusCode, response::IntoResponse, Json};
use tracing::error;
use uuid::Uuid;

pub async fn upload_file(mut multipart: Multipart) -> impl IntoResponse {
    // Expect a single field named "file"
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().map(|s| s.to_string());
        if name.as_deref() != Some("file") {
            continue;
        }
        let file_name = field.file_name().map(|s| s.to_string()).unwrap_or_else(|| "upload.bin".to_string());
        let bytes = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                error!(?e, "read multipart bytes failed");
                return StatusCode::BAD_REQUEST.into_response();
            }
        };

        // Ensure uploads dir exists
        let uploads_dir = std::env::var("UPLOADS_DIR").unwrap_or_else(|_| "uploads".into());
        let _ = tokio::fs::create_dir_all(&uploads_dir).await;

        let ext = std::path::Path::new(&file_name).extension().and_then(|s| s.to_str()).unwrap_or("");
        let unique = format!("{}{}{}",
            Uuid::new_v4(),
            if ext.is_empty() { "" } else { "." },
            ext
        );
        let mut path = PathBuf::from(&uploads_dir);
        path.push(&unique);
        match tokio::fs::write(&path, &bytes).await {
            Ok(_) => {
                let url = format!("/uploads/{}", unique);
                let body = serde_json::json!({"url": url});
                return (StatusCode::OK, Json(body)).into_response();
            }
            Err(e) => {
                error!(?e, "write upload failed");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    StatusCode::BAD_REQUEST.into_response()
}

