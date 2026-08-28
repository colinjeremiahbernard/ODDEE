use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{domain::event::PhysicalEvent, state::AppState};

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
