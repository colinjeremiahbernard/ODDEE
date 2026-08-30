use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{domain::anomaly::Anomaly, state::AppState};

pub async fn get_anomaly(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Anomaly>, (StatusCode, String)> {
    let anomaly = sqlx::query_as::<_, Anomaly>(
        r#"
        SELECT
            id,
            detected_at,
            entity_id,
            source_event_id,
            severity,
            reason,
            metadata
        FROM anomalies
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch anomaly: {error}"),
        )
    })?;

    match anomaly {
        Some(anomaly) => Ok(Json(anomaly)),
        None => Err((StatusCode::NOT_FOUND, format!("Anomaly not found: {id}"))),
    }
}
