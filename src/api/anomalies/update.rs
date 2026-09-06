use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{domain::anomaly::Anomaly, state::AppState};

const ALLOWED_STATUSES: [&str; 4] = ["new", "investigating", "resolved", "dismissed"];

#[derive(Debug, Deserialize)]
pub struct UpdateAnomalyStatusRequest {
    pub status: String,
}

/// PATCH /anomalies/{id} — update an anomaly's investigation status.
pub async fn update_anomaly_status(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateAnomalyStatusRequest>,
) -> Result<Json<Anomaly>, (StatusCode, String)> {
    if !ALLOWED_STATUSES.contains(&payload.status.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Invalid status '{}'. Must be one of: {}",
                payload.status,
                ALLOWED_STATUSES.join(", ")
            ),
        ));
    }

    let anomaly = sqlx::query_as::<_, Anomaly>(
        r#"
        UPDATE anomalies
        SET status = CAST($2 AS anomaly_status)
        WHERE id = $1
        RETURNING
            id,
            detected_at,
            severity,
            score,
            title,
            explanation,
            status::text AS status,
            related_event_ids
        "#,
    )
    .bind(id)
    .bind(&payload.status)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to update anomaly: {error}"),
        )
    })?;

    match anomaly {
        Some(anomaly) => Ok(Json(anomaly)),
        None => Err((StatusCode::NOT_FOUND, format!("Anomaly not found: {id}"))),
    }
}