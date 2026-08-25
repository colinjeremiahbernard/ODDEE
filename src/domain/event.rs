use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalEvent {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub source: EventSource,
    pub entity_id: String,
    pub kind: EventKind,
    pub zone: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSource {
    Api,
    Import,
    Sensor,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    Created,
    Updated,
    Deleted,
    Archived,
}
