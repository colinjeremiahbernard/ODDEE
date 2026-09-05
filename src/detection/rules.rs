use chrono::{Timelike, Utc};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

use crate::domain::{
    anomaly::Anomaly,
    event::{EventKind, PhysicalEvent},
};

/// A detected anomaly candidate returned by a rule before it is persisted.
pub struct DetectedAnomaly {
    pub title: String,
    pub explanation: String,
    pub severity: String,
    pub score: f32,
    pub related_event_ids: Vec<Uuid>,
    /// Optional evidence attachments (e.g., camera snapshot IDs).
    #[allow(dead_code)]
    pub snapshot_ids: Vec<String>,
}

impl DetectedAnomaly {
    pub async fn persist(self, pool: &PgPool) -> Result<Anomaly, sqlx::Error> {
        let anomaly = sqlx::query_as::<_, Anomaly>(
            r#"
            INSERT INTO anomalies (
                id,
                detected_at,
                severity,
                score,
                title,
                explanation,
                status,
                related_event_ids
            )
            VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS anomaly_status), $8)
            RETURNING
                id,
                detected_at,
                severity,
                score,
                title,
                explanation,
                status::text AS status,
                related_event_ids
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(Utc::now())
        .bind(self.severity)
        .bind(self.score)
        .bind(self.title)
        .bind(self.explanation)
        .bind("new") // anomaly_status
        .bind(&self.related_event_ids)
        .fetch_one(pool)
        .await?;

        Ok(anomaly)
    }
}

/// Rule 1: ObjectMissing without a preceding TransactionRecorded in the same zone
/// within the last 60 minutes → HIGH anomaly.
pub async fn object_missing_without_transaction(
    event: &PhysicalEvent,
    pool: &PgPool,
) -> Option<DetectedAnomaly> {
    if event.kind != EventKind::ObjectMissing {
        return None;
    }

    let window_start = event.occurred_at - Duration::from_secs(60 * 60);

    let found: Option<bool> = sqlx::query_scalar(
        r#"
        SELECT TRUE
        FROM physical_events
        WHERE entity_id = $1
          AND zone      = $2
          AND kind      = 'transaction_recorded'
          AND occurred_at BETWEEN $3 AND $4
        LIMIT 1
        "#,
    )
    .bind(&event.entity_id)
    .bind(&event.zone)
    .bind(window_start)
    .bind(event.occurred_at)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if found.is_some() {
        return None;
    }

    Some(DetectedAnomaly {
        title: "Object missing without transaction".to_string(),
        explanation: format!(
            "Entity '{}' went missing in zone '{}' with no inventory transaction in the prior 60 minutes.",
            event.entity_id, event.zone
        ),
        severity: "High".to_string(),
        score: 0.9,
        related_event_ids: vec![event.id],
        snapshot_ids: vec![],
    })
}

/// Rule 2: AccessDenied repeated 3+ times for the same entity within 10 minutes → CRITICAL.
pub async fn repeated_access_denied(
    event: &PhysicalEvent,
    pool: &PgPool,
) -> Option<DetectedAnomaly> {
    if event.kind != EventKind::AccessDenied {
        return None;
    }

    let window_start = event.occurred_at - Duration::from_secs(10 * 60);

    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM physical_events
        WHERE entity_id  = $1
          AND kind       = 'access_denied'
          AND occurred_at BETWEEN $2 AND $3
        "#,
    )
    .bind(&event.entity_id)
    .bind(window_start)
    .bind(event.occurred_at)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if count < 3 {
        return None;
    }

    Some(DetectedAnomaly {
        title: "Repeated access denied".to_string(),
        explanation: format!(
            "Entity '{}' had {} consecutive access denials within 10 minutes.",
            event.entity_id, count
        ),
        severity: "Critical".to_string(),
        score: 0.95,
        related_event_ids: vec![event.id],
        snapshot_ids: vec![],
    })
}

/// Rule 3: ObjectMoved in a zone outside operating hours (00:00 – 06:00 UTC) → MEDIUM.
pub async fn off_hours_movement(event: &PhysicalEvent, _pool: &PgPool) -> Option<DetectedAnomaly> {
    if event.kind != EventKind::ObjectMoved {
        return None;
    }

    let hour = event.occurred_at.time().hour();

    if hour >= 6 {
        return None;
    }

    Some(DetectedAnomaly {
        title: "Off-hours movement".to_string(),
        explanation: format!(
            "Entity '{}' was moved in zone '{}' during off-hours ({:02}:00 UTC).",
            event.entity_id, event.zone, hour
        ),
        severity: "Medium".to_string(),
        score: 0.7,
        related_event_ids: vec![event.id],
        snapshot_ids: vec![],
    })
}

/// Rule 4: Entity entered a zone it has never been seen in before → LOW.
pub async fn novel_zone_entry(event: &PhysicalEvent, pool: &PgPool) -> Option<DetectedAnomaly> {
    if event.kind != EventKind::EnteredZone {
        return None;
    }

    let prior_visits: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM physical_events
        WHERE entity_id = $1
          AND zone      = $2
          AND kind      = 'entered_zone'
          AND id       != $3
        "#,
    )
    .bind(&event.entity_id)
    .bind(&event.zone)
    .bind(event.id)
    .fetch_one(pool)
    .await
    .unwrap_or(1);

    if prior_visits > 0 {
        return None;
    }

    Some(DetectedAnomaly {
        title: "Novel zone entry".to_string(),
        explanation: format!(
            "Entity '{}' entered zone '{}' for the first time.",
            event.entity_id, event.zone
        ),
        severity: "Low".to_string(),
        score: 0.5,
        related_event_ids: vec![event.id],
        snapshot_ids: vec![],
    })
}

/// Rule 5: Repeated zone entry without exit within 15 minutes → HIGH.
///
/// If the same entity_id enters the same zone twice within a short window
/// without an exited_zone event in between, flag an anomaly.
pub async fn repeated_zone_entry(event: &PhysicalEvent, pool: &PgPool) -> Option<DetectedAnomaly> {
    if event.kind != EventKind::EnteredZone {
        return None;
    }

    let window_start = event.occurred_at - Duration::from_secs(15 * 60);

    // Fetch all entered_zone and exited_zone events for this entity+zone in the window
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT id, kind::text AS "kind!"
        FROM physical_events
        WHERE entity_id = $1
          AND zone      = $2
          AND occurred_at BETWEEN $3 AND $4
        ORDER BY occurred_at ASC
        "#,
    )
    .bind(&event.entity_id)
    .bind(&event.zone)
    .bind(window_start)
    .bind(event.occurred_at)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Count entered_zone events
    let entered_ids: Vec<Uuid> = rows
        .iter()
        .filter(|(_, kind)| kind == "entered_zone")
        .map(|(id, _)| *id)
        .collect();

    // Trigger if at least 2 entered_zone events in the window
    if entered_ids.len() < 2 {
        return None;
    }

    Some(DetectedAnomaly {
        title: "Repeated zone entry".to_string(),
        explanation: format!(
            "Entity '{}' entered zone '{}' {} times within a short window without an exit.",
            event.entity_id,
            event.zone,
            entered_ids.len()
        ),
        severity: "High".to_string(),
        score: 0.9,
        related_event_ids: entered_ids.clone(),
        snapshot_ids: vec![],
    })
}