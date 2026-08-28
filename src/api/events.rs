use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{Postgres, QueryBuilder, types::Json as SqlxJson};
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

#[derive(Debug, Deserialize)]
pub struct EventFilters {
    pub entity_id: Option<String>,
    pub zone: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_event(
    State(state): State<AppState>,
    Json(payload): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<PhysicalEvent>), (StatusCode, String)> {
    if payload.entity_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "entity_id must not be empty".to_string(),
        ));
    }

    if payload.zone.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "zone must not be empty".to_string(),
        ));
    }

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
    Query(filters): Query<EventFilters>,
) -> Result<Json<Vec<PhysicalEvent>>, (StatusCode, String)> {
    let limit = filters.limit.unwrap_or(50).clamp(1, 100);
    let offset = filters.offset.unwrap_or(0).max(0);

    let mut query = QueryBuilder::<Postgres>::new(
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
        WHERE 1 = 1
        "#,
    );

    if let Some(entity_id) = filters.entity_id {
        query.push(" AND entity_id = ").push_bind(entity_id);
    }

    if let Some(zone) = filters.zone {
        query.push(" AND zone = ").push_bind(zone);
    }

    query
        .push(" ORDER BY occurred_at DESC, id DESC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let events = query
        .build_query_as::<PhysicalEvent>()
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
