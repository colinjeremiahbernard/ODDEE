use axum::{Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::types::Json as SqlxJson;
use uuid::Uuid;

use crate::{
    domain::anomaly::{Anomaly, AnomalySeverity},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateAnomalyRequest {
    pub detected_at: DateTime<Utc>,
    pub entity_id: String,
    pub source_event_id: Option<Uuid>,
    pub severity: AnomalySeverity,
    pub reason: String,
    pub metadata: serde_json::Value,
}

pub async fn create_anomaly(
    State(state): State<AppState>,
    Json(payload): Json<CreateAnomalyRequest>,
) -> Result<Json<Anomaly>, (StatusCode, String)> {
    let anomaly_id = Uuid::new_v4();

    let anomaly = sqlx::query_as::<_, Anomaly>(
        r#"
        INSERT INTO anomalies (
            id,
            detected_at,
            entity_id,
            source_event_id,
            severity,
            reason,
            metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id,
            detected_at,
            entity_id,
            source_event_id,
            severity,
            reason,
            metadata
        "#,
    )
    .bind(anomaly_id)
    .bind(payload.detected_at)
    .bind(payload.entity_id)
    .bind(payload.source_event_id)
    .bind(payload.severity)
    .bind(payload.reason)
    .bind(SqlxJson(payload.metadata))
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
