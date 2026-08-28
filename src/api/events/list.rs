use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::QueryBuilder;

use crate::{domain::event::PhysicalEvent, state::AppState};

#[derive(Debug, Deserialize)]
pub struct EventFilters {
    pub entity_id: Option<String>,
    pub zone: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_events(
    State(state): State<AppState>,
    Query(filters): Query<EventFilters>,
) -> Result<Json<Vec<PhysicalEvent>>, (StatusCode, String)> {
    let limit = filters.limit.unwrap_or(50).clamp(1, 100);
    let offset = filters.offset.unwrap_or(0).max(0);

    let mut query = QueryBuilder::<sqlx::Postgres>::new(
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
