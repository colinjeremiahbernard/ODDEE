// src/api/events.rs
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    domain::event::{EventKind, EventSource, PhysicalEvent},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub occurred_at: chrono::DateTime<chrono::Utc>,
    pub source: EventSource,
    pub entity_id: String,
    pub kind: EventKind,
    pub zone: String,
    pub metadata: serde_json::Value,
}

pub async fn create_event(
    State(_state): State<AppState>,
    Json(payload): Json<CreateEventRequest>,
) -> Result<Json<PhysicalEvent>, (StatusCode, String)> {
    let event = PhysicalEvent {
        id: Uuid::new_v4(),
        occurred_at: payload.occurred_at,
        source: payload.source,
        entity_id: payload.entity_id,
        kind: payload.kind,
        zone: payload.zone,
        metadata: payload.metadata,
    };

    Ok(Json(event))
}
