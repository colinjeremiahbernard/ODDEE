//! Deterministic event replay engine.
//!
//! Posts a fixed, reproducible sequence of physical events to a running
//! ODDEE backend via `POST /events`, exercising the real ingestion and
//! detection pipeline exactly as a live system would. Run against a
//! freshly migrated (empty) database for fully reproducible output —
//! every entity's first zone visit is expected to trigger a low-severity
//! "novel zone entry" anomaly, which is correct detection behavior, not
//! an artifact of the replay.
//!
//! The scenario intentionally re-creates the exact case from the
//! README's "Example Alert": package `BX-2041` disappearing from a
//! normally static shelf with no matching inventory transaction. It
//! also exercises every other rule in `detection::rules` so a full demo
//! run produces at least one anomaly of each kind.
//!
//! Usage:
//!   cargo run --bin replay
//!   ODDEE_API_URL=http://localhost:3000 cargo run --bin replay -- --delay-ms 250

use chrono::{DateTime, TimeZone, Utc};
use serde_json::{json, Value};
use std::time::Duration;

/// Fixed scenario date. Keeping this constant (rather than "now") is what
/// makes the replay deterministic: the same scenario produces byte-for-byte
/// identical anomaly explanations on every run against a clean database.
fn scenario_date() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 15, 0, 0, 0).unwrap()
}

struct ScenarioEvent {
    label: &'static str,
    offset_secs: i64,
    /// Absolute override for events that must land on a specific
    /// wall-clock hour (e.g. the off-hours rule checks the literal UTC
    /// hour of `occurred_at`, not an offset from the scenario start).
    absolute: Option<DateTime<Utc>>,
    source: &'static str,
    entity_id: &'static str,
    kind: &'static str,
    zone: &'static str,
    metadata: Value,
}

fn scenario() -> Vec<ScenarioEvent> {
    let base = scenario_date();

    vec![
        // --- Normal baseline activity (unremarkable warehouse operation) ---
        ScenarioEvent {
            label: "Forklift enters loading dock",
            offset_secs: 0,
            absolute: None,
            source: "sensor",
            entity_id: "forklift-07",
            kind: "entered_zone",
            zone: "loading-dock",
            metadata: json!({}),
        },
        ScenarioEvent {
            label: "Forklift exits loading dock",
            offset_secs: 120,
            absolute: None,
            source: "sensor",
            entity_id: "forklift-07",
            kind: "exited_zone",
            zone: "loading-dock",
            metadata: json!({}),
        },
        ScenarioEvent {
            label: "Worker enters packing area",
            offset_secs: 150,
            absolute: None,
            source: "access_control",
            entity_id: "worker-12",
            kind: "entered_zone",
            zone: "packing",
            metadata: json!({}),
        },
        ScenarioEvent {
            label: "Worker exits packing area",
            offset_secs: 600,
            absolute: None,
            source: "access_control",
            entity_id: "worker-12",
            kind: "exited_zone",
            zone: "packing",
            metadata: json!({}),
        },
        // Package BX-2041 is placed on its normally static shelf, with a
        // matching inventory transaction — this is the "expected" baseline
        // the later anomaly will violate.
        ScenarioEvent {
            label: "Package BX-2041 placed on shelf B-12",
            offset_secs: 700,
            absolute: None,
            source: "camera",
            entity_id: "package-BX-2041",
            kind: "entered_zone",
            zone: "shelf-b12",
            metadata: json!({ "camera_id": "cam-04" }),
        },
        ScenarioEvent {
            label: "Inventory transaction recorded for BX-2041",
            offset_secs: 710,
            absolute: None,
            source: "inventory",
            entity_id: "package-BX-2041",
            kind: "transaction_recorded",
            zone: "shelf-b12",
            metadata: json!({ "transaction_type": "placement" }),
        },
        ScenarioEvent {
            label: "Second forklift makes a routine pass through zone A",
            offset_secs: 1800,
            absolute: None,
            source: "sensor",
            entity_id: "forklift-03",
            kind: "entered_zone",
            zone: "zone-a",
            metadata: json!({}),
        },
        ScenarioEvent {
            label: "Second forklift exits zone A",
            offset_secs: 1900,
            absolute: None,
            source: "sensor",
            entity_id: "forklift-03",
            kind: "exited_zone",
            zone: "zone-a",
            metadata: json!({}),
        },
        // --- THE headline anomaly: README's own "Example Alert" ---
        // BX-2041 disappears from Shelf B-12 more than 60 minutes after its
        // last recorded transaction, with no approved workflow behind it.
        // Triggers `object_missing_without_transaction` → HIGH.
        ScenarioEvent {
            label: "*** BX-2041 goes missing from Shelf B-12 (headline anomaly) ***",
            offset_secs: 6 * 3600 + 32 * 60, // 14:32, matching the README example
            absolute: None,
            source: "camera",
            entity_id: "package-BX-2041",
            kind: "object_missing",
            zone: "shelf-b12",
            metadata: json!({ "camera_id": "cam-04" }),
        },
        // --- Repeated zone entry without exit → HIGH ---
        ScenarioEvent {
            label: "Crate CR-118 enters zone A",
            offset_secs: 6 * 3600 + 40 * 60,
            absolute: None,
            source: "camera",
            entity_id: "crate-CR-118",
            kind: "entered_zone",
            zone: "zone-a",
            metadata: json!({}),
        },
        ScenarioEvent {
            label: "Crate CR-118 enters zone A again, no exit in between",
            offset_secs: 6 * 3600 + 45 * 60,
            absolute: None,
            source: "camera",
            entity_id: "crate-CR-118",
            kind: "entered_zone",
            zone: "zone-a",
            metadata: json!({}),
        },
        // --- Off-hours movement → MEDIUM ---
        // The rule checks the literal UTC hour of `occurred_at`, so this
        // event uses an absolute timestamp rather than a scenario offset.
        ScenarioEvent {
            label: "Machine moved at 03:15 UTC (off-hours)",
            offset_secs: 0,
            absolute: Some(Utc.with_ymd_and_hms(2026, 1, 15, 3, 15, 0).unwrap()),
            source: "machine",
            entity_id: "conveyor-02",
            kind: "object_moved",
            zone: "zone-c",
            metadata: json!({}),
        },
        // --- Repeated access denied (3x within 10 min) → CRITICAL ---
        ScenarioEvent {
            label: "Badge 099 denied access to server room (1/3)",
            offset_secs: 7 * 3600,
            absolute: None,
            source: "access_control",
            entity_id: "badge-099",
            kind: "access_denied",
            zone: "server-room",
            metadata: json!({}),
        },
        ScenarioEvent {
            label: "Badge 099 denied access to server room (2/3)",
            offset_secs: 7 * 3600 + 120,
            absolute: None,
            source: "access_control",
            entity_id: "badge-099",
            kind: "access_denied",
            zone: "server-room",
            metadata: json!({}),
        },
        ScenarioEvent {
            label: "Badge 099 denied access to server room (3/3)",
            offset_secs: 7 * 3600 + 240,
            absolute: None,
            source: "access_control",
            entity_id: "badge-099",
            kind: "access_denied",
            zone: "server-room",
            metadata: json!({}),
        },
    ]
    .into_iter()
    .map(|mut e| {
        if e.absolute.is_none() {
            e.absolute = Some(base + chrono::Duration::seconds(e.offset_secs));
        }
        e
    })
    .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_url = std::env::var("ODDEE_API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let delay_ms: u64 = std::env::args()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|w| w[0] == "--delay-ms")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(150);

    let client = reqwest::Client::new();
    let events = scenario();

    println!("ODDEE replay engine");
    println!("Target API: {api_url}");
    println!("Events in scenario: {}", events.len());
    println!("---");

    for (i, event) in events.iter().enumerate() {
        let occurred_at = event.absolute.expect("resolved above");

        let body = json!({
            "occurred_at": occurred_at.to_rfc3339(),
            "source": event.source,
            "entity_id": event.entity_id,
            "kind": event.kind,
            "zone": event.zone,
            "metadata": event.metadata,
        });

        let response = client
            .post(format!("{api_url}/events"))
            .json(&body)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                println!("[{:02}/{}] OK   {}", i + 1, events.len(), event.label);
            }
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                eprintln!(
                    "[{:02}/{}] FAIL ({status}) {} — {text}",
                    i + 1,
                    events.len(),
                    event.label
                );
            }
            Err(err) => {
                eprintln!(
                    "[{:02}/{}] ERROR {} — {err}",
                    i + 1,
                    events.len(),
                    event.label
                );
                eprintln!("Is the ODDEE backend running at {api_url}?");
                return Err(err.into());
            }
        }

        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    println!("---");
    println!("Replay complete. Detection runs asynchronously per event,");
    println!("so give it a moment, then check:");
    println!("  {api_url}/anomalies");
    println!("or open the dashboard at http://localhost:4200");
    println!();
    println!("Expect at least one anomaly of each kind: novel zone entry,");
    println!("object missing without transaction, repeated zone entry,");
    println!("off-hours movement, and repeated access denied.");

    Ok(())
}