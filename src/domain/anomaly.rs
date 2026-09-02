use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Anomaly {
    pub id: Uuid,
    pub detected_at: DateTime<Utc>,
    pub severity: String,
    pub score: f32,
    pub title: String,
    pub explanation: String,
    pub status: String,
    pub related_event_ids: Vec<Uuid>,
}
