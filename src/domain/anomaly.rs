use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "anomaly_severity", rename_all = "snake_case")]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Anomaly {
    pub id: Uuid,
    pub detected_at: DateTime<Utc>,
    pub entity_id: String,
    pub source_event_id: Option<Uuid>,
    pub severity: AnomalySeverity,
    pub reason: String,
    pub metadata: serde_json::Value,
}
