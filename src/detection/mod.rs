pub mod rules;

use sqlx::PgPool;
use tracing::{error, info};

use crate::domain::event::PhysicalEvent;

use rules::{
    novel_zone_entry, object_missing_without_transaction, off_hours_movement,
    repeated_access_denied, repeated_zone_entry,
};

/// Run all detection rules against the newly ingested event.
/// Each rule returns `Some(DetectedAnomaly)` if it fires.
/// Fired anomalies are persisted immediately.
///
/// Failures in individual rules are logged but do not abort the others or the HTTP response.
pub async fn run_all(event: &PhysicalEvent, pool: &PgPool) {
    info!(
        event_id = %event.id,
        entity_id = %event.entity_id,
        kind = ?event.kind,
        "Running detection rules"
    );

    let rule_futures: Vec<
        std::pin::Pin<Box<dyn std::future::Future<Output = Option<rules::DetectedAnomaly>> + Send>>,
    > = vec![
        Box::pin(object_missing_without_transaction(event, pool)),
        Box::pin(repeated_access_denied(event, pool)),
        Box::pin(off_hours_movement(event, pool)),
        Box::pin(novel_zone_entry(event, pool)),
        Box::pin(repeated_zone_entry(event, pool)),
    ];

    let results = futures::future::join_all(rule_futures).await;

    for candidate in results.into_iter().flatten() {
        let title = candidate.title.clone();
        match candidate.persist(pool).await {
            Ok(anomaly) => {
                info!(
                    anomaly_id = %anomaly.id,
                    severity   = %anomaly.severity,
                    title = %anomaly.title,
                    "Anomaly detected and persisted"
                );
            }
            Err(err) => {
                error!(
                    title = %title,
                    error  = %err,
                    "Failed to persist detected anomaly"
                );
            }
        }
    }
}
