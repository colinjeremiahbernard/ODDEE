use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::types::Json as SqlxJson;
use uuid::Uuid;

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

pub async fn get_events(
    State(state): State<AppState>,
) -> Result<Json<Vec<PhysicalEvent>>, (StatusCode, String)> {
    let events = sqlx::query_as::<_, PhysicalEvent>(
        r#"
        SELECT
            id,
            occurred_at,
            source,
            entity_id,
            kind,
            zone,
            metadata
        FROM physical_events
        ORDER BY occurred_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch physical events: {error}"),
        )
    })?;

    Ok(Json(events))
}

pub async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PhysicalEvent>, (StatusCode, String)> {
    let event = sqlx::query_as::<_, PhysicalEvent>(
        r#"
        SELECT
            id,
            occurred_at,
            source,
            entity_id,
            kind,
            zone,
            metadata
        FROM physical_events
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch physical event: {error}"),
        )
    })?;

    match event {
        Some(event) => Ok(Json(event)),
        None => Err((
            StatusCode::NOT_FOUND,
            format!("Physical event {id} not found"),
        )),
    }
}
