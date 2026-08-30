use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;

use crate::{domain::anomaly::Anomaly, state::AppState};

#[derive(Debug, Deserialize)]
pub struct AnomalyFilters {
    pub entity_id: Option<String>,
    pub severity: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedAnomaliesResponse {
    pub items: Vec<Anomaly>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

pub async fn get_anomalies(
    State(state): State<AppState>,
    Query(filters): Query<AnomalyFilters>,
) -> Result<Json<PaginatedAnomaliesResponse>, (StatusCode, String)> {
    let limit = filters.limit.unwrap_or(50).clamp(1, 100);
    let offset = filters.offset.unwrap_or(0).max(0);

    let mut items_query = QueryBuilder::<sqlx::Postgres>::new(
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
        WHERE 1 = 1
        "#,
    );

    if let Some(entity_id) = filters.entity_id.as_ref() {
        items_query.push(" AND entity_id = ").push_bind(entity_id);
    }

    if let Some(severity) = filters.severity.as_ref() {
        items_query.push(" AND severity = ").push_bind(severity);
    }

    items_query
        .push(" ORDER BY detected_at DESC, id DESC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let items = items_query
        .build_query_as::<Anomaly>()
        .fetch_all(&state.pool)
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch anomalies: {error}"),
            )
        })?;

    let mut count_query = QueryBuilder::<sqlx::Postgres>::new(
        r#"
        SELECT COUNT(*)
        FROM anomalies
        WHERE 1 = 1
        "#,
    );

    if let Some(entity_id) = filters.entity_id.as_ref() {
        count_query.push(" AND entity_id = ").push_bind(entity_id);
    }

    if let Some(severity) = filters.severity.as_ref() {
        count_query.push(" AND severity = ").push_bind(severity);
    }

    let total: i64 = count_query
        .build_query_scalar()
        .fetch_one(&state.pool)
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to count anomalies: {error}"),
            )
        })?;

    let has_more = offset + (items.len() as i64) < total;

    Ok(Json(PaginatedAnomaliesResponse {
        items,
        total,
        limit,
        offset,
        has_more,
    }))
}
