use axum::Json;

use crate::models::OpenNoticesResponse;

pub async fn list() -> Json<OpenNoticesResponse> {
    tracing::info!(notice_count = 0, "open notices requested");
    Json(OpenNoticesResponse {
        code: 0,
        notices: Vec::new(),
    })
}
