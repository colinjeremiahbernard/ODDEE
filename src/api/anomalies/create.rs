use axum::{Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{domain::anomaly::Anomaly, state::AppState};

#[derive(Debug, Deserialize)]
pub struct CreateAnomalyRequest {
    pub detected_at: DateTime<Utc>,
    pub severity: String,
    pub score: f32,
    pub title: String,
    pub explanation: String,
    pub status: Option<String>,
    pub related_event_ids: Vec<Uuid>,
}

pub async fn create_anomaly(
    State(state): State<AppState>,
    Json(payload): Json<CreateAnomalyRequest>,
) -> Result<Json<Anomaly>, (StatusCode, String)> {
    let anomaly_id = Uuid::new_v4();

    let status = payload.status.unwrap_or_else(|| "new".to_string());

    let anomaly = sqlx::query_as::<_, Anomaly>(
        r#"
        INSERT INTO anomalies (
            id,
            detected_at,
            severity,
            score,
            title,
            explanation,
            status,
            related_event_ids
        )
        VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS anomaly_status), $8)
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
    .bind(anomaly_id)
    .bind(payload.detected_at)
    .bind(payload.severity)
    .bind(payload.score)
    .bind(payload.title)
    .bind(payload.explanation)
    .bind(status)
    .bind(&payload.related_event_ids)
    .fetch_one(&state.pool)
    .await
    .map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create anomaly: {error}"),
        )
    })?;

    Ok(Json(anomaly))
}
