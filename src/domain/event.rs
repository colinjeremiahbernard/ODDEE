use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "event_source", rename_all = "snake_case")]
pub enum EventSource {
    Camera,
    Sensor,
    Inventory,
    AccessControl,
    Machine,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "event_kind", rename_all = "snake_case")]
pub enum EventKind {
    EnteredZone,
    ExitedZone,
    ObjectMoved,
    ObjectMissing,
    StateChanged,
    AccessGranted,
    AccessDenied,
    TransactionRecorded,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PhysicalEvent {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub source: EventSource,
    pub entity_id: String,
    pub kind: EventKind,
    pub zone: String,
    pub metadata: serde_json::Value,
}
