use axum::{Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::types::Json as SqlxJson;

use crate::{
    domain::event::{EventKind, EventSource, PhysicalEvent},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub occurred_at: DateTime<Utc>,
    pub source: EventSource,
    pub entity_id: String,
    pub kind: EventKind,
    pub zone: String,
    pub metadata: serde_json::Value,
}

pub async fn create_event(
    State(state): State<AppState>,
    Json(payload): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<PhysicalEvent>), (StatusCode, String)> {
    let event = sqlx::query_as::<_, PhysicalEvent>(
        r#"
        INSERT INTO physical_events (
            occurred_at,
            source,
            entity_id,
            kind,
            zone,
            metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id,
            occurred_at,
            source,
            entity_id,
            kind,
            zone,
            metadata
        "#,
    )
    .bind(payload.occurred_at)
    .bind(payload.source)
    .bind(payload.entity_id)
    .bind(payload.kind)
    .bind(payload.zone)
    .bind(SqlxJson(payload.metadata))
    .fetch_one(&state.pool)
    .await
    .map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create physical event: {error}"),
        )
    })?;

    Ok((StatusCode::CREATED, Json(event)))
}
