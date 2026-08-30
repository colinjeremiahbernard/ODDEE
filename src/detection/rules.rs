use chrono::{Duration, Timelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    anomaly::{Anomaly, AnomalySeverity},
    event::{EventKind, PhysicalEvent},
};

/// A detected anomaly candidate returned by a rule before it is persisted.
pub struct DetectedAnomaly {
    pub entity_id: String,
    pub source_event_id: Option<Uuid>,
    pub severity: AnomalySeverity,
    pub reason: String,
    pub metadata: serde_json::Value,
}

impl DetectedAnomaly {
    pub async fn persist(self, pool: &PgPool) -> Result<Anomaly, sqlx::Error> {
        let anomaly = sqlx::query_as::<_, Anomaly>(
            r#"
            INSERT INTO anomalies (
                id,
                detected_at,
                entity_id,
                source_event_id,
                severity,
                reason,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id,
                detected_at,
                entity_id,
                source_event_id,
                severity,
                reason,
                metadata
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(Utc::now())
        .bind(&self.entity_id)
        .bind(self.source_event_id)
        .bind(&self.severity)
        .bind(&self.reason)
        .bind(sqlx::types::Json(&self.metadata))
        .fetch_one(pool)
        .await?;

        Ok(anomaly)
    }
}

/// Rule 1: ObjectMissing without a preceding TransactionRecorded in the same zone
/// within the last 60 minutes → HIGH anomaly.
///
/// An object that disappears without any inventory transaction is suspicious.
pub async fn object_missing_without_transaction(
    event: &PhysicalEvent,
    pool: &PgPool,
) -> Option<DetectedAnomaly> {
    if event.kind != EventKind::ObjectMissing {
        return None;
    }

    let window_start = event.occurred_at - Duration::minutes(60);

    // Look for a TransactionRecorded in the same zone for the same entity in the time window
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
        // There was a transaction — expected, no anomaly
        return None;
    }

    Some(DetectedAnomaly {
        entity_id: event.entity_id.clone(),
        source_event_id: Some(event.id),
        severity: AnomalySeverity::High,
        reason: format!(
            "Entity '{}' went missing in zone '{}' with no inventory transaction in the prior 60 minutes.",
            event.entity_id, event.zone
        ),
        metadata: serde_json::json!({
            "zone": event.zone,
            "event_kind": "object_missing",
            "window_minutes": 60
        }),
    })
}

/// Rule 2: AccessDenied repeated 3+ times for the same entity within 10 minutes → CRITICAL.
///
/// Repeated access denials suggest a brute-force or tailgating attempt.
pub async fn repeated_access_denied(
    event: &PhysicalEvent,
    pool: &PgPool,
) -> Option<DetectedAnomaly> {
    if event.kind != EventKind::AccessDenied {
        return None;
    }

    let window_start = event.occurred_at - Duration::minutes(10);

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

    // The current event is already persisted before detection runs, so count >= 3
    if count < 3 {
        return None;
    }

    // Only fire on the 3rd hit (not every subsequent one) to avoid duplicate anomalies
    if count != 3 {
        return None;
    }

    Some(DetectedAnomaly {
        entity_id: event.entity_id.clone(),
        source_event_id: Some(event.id),
        severity: AnomalySeverity::Critical,
        reason: format!(
            "Entity '{}' had {} consecutive access denials within 10 minutes.",
            event.entity_id, count
        ),
        metadata: serde_json::json!({
            "denial_count": count,
            "window_minutes": 10,
            "zone": event.zone
        }),
    })
}

/// Rule 3: ObjectMoved in a zone outside operating hours (00:00 – 06:00 UTC) → MEDIUM.
///
/// Movement during off-hours in a monitored facility is unexpected.
pub async fn off_hours_movement(
    event: &PhysicalEvent,
    _pool: &PgPool,
) -> Option<DetectedAnomaly> {
    if event.kind != EventKind::ObjectMoved {
        return None;
    }

    let hour = event.occurred_at.time().hour();

    // Off-hours: midnight to 6 AM UTC
    if hour >= 6 {
        return None;
    }

    Some(DetectedAnomaly {
        entity_id: event.entity_id.clone(),
        source_event_id: Some(event.id),
        severity: AnomalySeverity::Medium,
        reason: format!(
            "Entity '{}' was moved in zone '{}' during off-hours ({:02}:00 UTC).",
            event.entity_id, event.zone, hour
        ),
        metadata: serde_json::json!({
            "hour_utc": hour,
            "zone": event.zone,
            "event_kind": "object_moved"
        }),
    })
}

/// Rule 4: Entity entered a zone it has never been seen in before → LOW.
///
/// Novel zone access may indicate misconfiguration or unauthorized entry.
pub async fn novel_zone_entry(
    event: &PhysicalEvent,
    pool: &PgPool,
) -> Option<DetectedAnomaly> {
    if event.kind != EventKind::EnteredZone {
        return None;
    }

    // Count prior visits to this zone by this entity (excluding the current event)
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
    .unwrap_or(1); // default to 1 so we don't fire if the query fails

    if prior_visits > 0 {
        return None;
    }

    Some(DetectedAnomaly {
        entity_id: event.entity_id.clone(),
        source_event_id: Some(event.id),
        severity: AnomalySeverity::Low,
        reason: format!(
            "Entity '{}' entered zone '{}' for the first time.",
            event.entity_id, event.zone
        ),
        metadata: serde_json::json!({
            "zone": event.zone,
            "first_visit": true
        }),
    })
}
