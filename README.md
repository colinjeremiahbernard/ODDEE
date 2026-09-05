# ODDEƎ

> When reality gets odd, ODDEE knows.

ODDEƎ is an AI-powered physical-world anomaly intelligence platform that learns how an environment normally behaves and identifies events that do not belong. Instead of merely recognizing objects, ODDEƎ compares expected behavior with observed reality across cameras, sensors, inventory systems, access-control systems, and machine data. Its purpose is to detect unusual movement, missing objects, unexpected sequences, configuration changes, and physical events that conflict with digital records, then explain the evidence clearly enough for a human to investigate.

## Project Status

This project is currently a focused prototype for a warehouse environment. The first implementation demonstrates how ODDEƎ can learn a simplified baseline of normal warehouse activity, detect an unexpected package movement, correlate that event with inventory data, and present an understandable anomaly alert.

## Core Concept

ODDEƎ does not ask only:

> What is in this scene?

It asks:

> What should be happening here, and why does reality disagree?

The system models normal behavior for a physical environment, including object locations, movement routes, timing, process sequences, machine states, and relationships between physical and digital events. When observed activity violates the learned expectation, ODDEƎ creates an anomaly record containing a confidence score, explanation, related evidence, and suggested next action.

## Prototype Scenario

The initial prototype focuses on a warehouse where packages, shelves, forklifts, workers, cameras, inventory transactions, and movement routes follow established patterns.

The demonstration detects an event such as a package moving from a normally static shelf without a corresponding inventory transaction or approved workflow. Instead of reporting only that a package was detected, ODDEƎ explains why the event is unusual:

- The package moved from a normally static location.
- No matching inventory transaction was recorded.
- No approved task corresponds to the movement.
- The observed route differs from the normal handling pattern.
- Camera, sensor, and inventory evidence refer to the same event.

## Example Alert

**That’s odd.**

Package `BX-2041` moved from Shelf B-12 at 14:32.

**Why ODDEƎ flagged it:**

- Shelf B-12 is normally static.
- No inventory transaction was recorded.
- No approved workflow matches the movement.
- The observed route differs from the normal handling pattern.

**Evidence:** Camera 04, Zone B sensors, and inventory events  
**Confidence:** High  
**Status:** Needs review

## Planned Features

- Physical-environment event ingestion.
- Simulated camera and sensor event streams.
- Warehouse baseline modeling.
- Expected package locations and movement routes.
- Physical-event and digital-event correlation.
- Anomaly scoring.
- Natural-language anomaly explanations.
- Evidence timelines.
- Anomaly review and investigation status.
- Resolved, dismissed, and escalated alert states.
- Reproducible event replay for demonstrations and testing.

## Initial Architecture

The prototype is designed around the following components:

- **Frontend:** A visual dashboard showing the warehouse layout, live events, anomaly markers, and evidence timeline. The current prototype focuses on the backend API and detection layer; a frontend will be added in a subsequent phase.
- **Backend:** A Rust API responsible for event ingestion, baseline evaluation, anomaly generation, and alert management.
- **Database:** PostgreSQL for physical events, expected patterns, anomalies, evidence relationships, and investigation state.
- **Replay engine:** A deterministic simulator that produces normal and anomalous warehouse events.
- **Detection layer:** Rule-based baseline detection for the first implementation, with an AI explanation layer added after the core behavior is reliable.
- **Deployment:** Docker Compose for local development and repeatable demonstrations.

## Suggested Event Model

```rust
struct PhysicalEvent {
    id: Uuid,
    timestamp: DateTime<Utc>,
    source: EventSource,
    entity_id: String,
    event_type: EventType,
    location: Location,
    metadata: serde_json::Value,
}

enum EventSource {
    Camera,
    Sensor,
    Inventory,
    AccessControl,
    Machine,
}

enum EventType {
    EnteredZone,
    ExitedZone,
    ObjectMoved,
    ObjectMissing,
    StateChanged,
    AccessGranted,
    AccessDenied,
    TransactionRecorded,
}
```

## Suggested Anomaly Model

```rust
struct Anomaly {
    id: Uuid,
    severity: Severity,
    score: f32,
    title: String,
    explanation: String,
    related_event_ids: Vec<Uuid>,
    status: AnomalyStatus,
}
```

## Brand Identity

The official plain-text name is **ODDEE**. The display wordmark is **ODDEƎ**, with only the final E reversed. The reversed E represents the anomaly: one element violates an otherwise familiar pattern, just as ODDEƎ detects deviations in physical environments.

The name is pronounced **“oddie.”**

### Brand personality

- Observant.
- Precise.
- Calm.
- Unsettling.
- Technically credible.
- Approachable.

ODDEƎ should feel like a quiet investigator rather than an aggressive surveillance system. It should report unusual conditions clearly, explain the evidence, and preserve human judgment.

### Brand language

Preferred phrases include:

- “That’s odd.”
- “Something changed.”
- “Reality and the system disagree.”
- “This event does not match the expected pattern.”
- “One detail changed. Here is the evidence.”
- “ODDEƎ found a deviation.”

Avoid unsupported claims such as:

- “Perfect security.”
- “Zero false positives.”
- “The AI knows everything.”
- “Guaranteed prevention.”
- “Threat eliminated.”

## Visual Direction

The visual principle is:

> Everything is orderly until ODDEƎ finds the one thing that is not.

The visual system should use orderly grids, clean geometry, calm surfaces, and one displaced or interrupted element to represent an anomaly.

### Color palette

| Role             | Color           | Hex       |
| ---------------- | --------------- | --------- |
| Main text        | Graphite black  | `#17191C` |
| Background       | Warm white      | `#F6F5F0` |
| Anomaly accent   | Electric orange | `#FF6B35` |
| Secondary signal | Acid yellow     | `#D7F54A` |
| Supporting data  | Cool blue       | `#73B7D8` |
| Muted interface  | Slate gray      | `#69727D` |

The anomaly accent should be used sparingly so that detected deviations become visually important.

## First Milestone

The first implementation milestone is a polished warehouse anomaly demonstration containing:

1. A warehouse floor-plan view.
2. Normal package and forklift movement.
3. A simulated package movement that violates the baseline.
4. An event timeline.
5. A visual anomaly marker.
6. A generated anomaly alert.
7. Evidence linking the camera, sensor, and inventory events.
8. A control for marking the anomaly as resolved.

The prototype should prove one idea clearly: ODDEƎ can distinguish an ordinary object from an unusual event involving that object.

## Development Setup

### Prerequisites

Install the following tools before running the project:

- Rust and Cargo.
- Docker Desktop.
- Docker Compose.
- PostgreSQL, if running the database outside Docker.
- Node.js, if the frontend uses a JavaScript framework.

### Clone the repository

```bash
git clone <repository-url>
cd <repository-directory>
```

### Start supporting services

```bash
docker compose up -d
```

### Run database migrations

```bash
cargo run --bin migrate
```

### Start the backend

```bash
cargo run
```

### Start the frontend

```bash
npm install
npm run dev
```

## API Reference

ODDEƎ exposes a REST API for ingesting physical events and querying detected anomalies. All examples below assume the backend is running locally on `http://localhost:3000`.

### Events

#### `POST /events`

Create a new physical event.

**Request body:**

```json
{
  "occurred_at": "2026-09-04T22:00:00Z",
  "source": "camera",
  "entity_id": "package-BX-2041",
  "kind": "entered_zone",
  "zone": "warehouse-a",
  "metadata": {
    "camera_id": "cam-01",
    "confidence": 0.96
  }
}
```

**Example (PowerShell):**

```powershell
$body = @{
    occurred_at = (Get-Date).ToUniversalTime().ToString("o")
    source      = "camera"
    entity_id   = "package-BX-2041"
    kind        = "entered_zone"
    zone        = "warehouse-a"
    metadata    = @{ camera_id = "cam-01"; confidence = 0.96 }
} | ConvertTo-Json

Invoke-RestMethod -Method Post -Uri "http://localhost:3000/events" `
    -Body $body -ContentType "application/json"
```

**Response:** `201 Created` with the created `PhysicalEvent`.

---

#### `GET /events`

List physical events with optional filters and pagination.

**Query parameters:**

- `entity_id` (optional): filter by entity
- `zone` (optional): filter by zone
- `limit` (optional, default 50, max 100)
- `offset` (optional, default 0)

**Example:**

```powershell
Invoke-RestMethod -Method Get `
    -Uri "http://localhost:3000/events?entity_id=package-BX-2041&limit=10"
```

**Response:** `200 OK` with a paginated envelope:

```json
{
  "items": [
    /* PhysicalEvent[] */
  ],
  "total": 42,
  "limit": 10,
  "offset": 0,
  "has_more": true
}
```

---

### Anomalies

#### `GET /anomalies`

List detected anomalies with optional filters and pagination.

**Query parameters:**

- `severity` (optional): e.g. `Low`, `Medium`, `High`, `Critical`
- `status` (optional): e.g. `new`, `acknowledged`, `resolved`
- `limit` (optional, default 50, max 100)
- `offset` (optional, default 0)

**Example:**

```powershell
Invoke-RestMethod -Method Get `
    -Uri "http://localhost:3000/anomalies?severity=High&limit=20"
```

**Response:** `200 OK` with a paginated envelope:

```json
{
  "items": [
    {
      "id": "57561b04-e4bd-401f-81b6-4aba9e8c67e1",
      "detected_at": "2026-09-04T22:10:00Z",
      "severity": "High",
      "score": 0.9,
      "title": "Repeated zone entry",
      "explanation": "Entity 'package-BX-2041' entered zone 'warehouse-a' 2 times within a short window without an exit.",
      "status": "new",
      "related_event_ids": [
        "0dccd63e-3fe3-4136-83d5-5ff80ea58d8d",
        "a204d572-5936-44f1-bab8-22fe2fb22dd2"
      ]
    }
  ],
  "total": 3,
  "limit": 20,
  "offset": 0,
  "has_more": false
}
```

Anomalies are generated automatically by background detection rules (e.g., repeated zone entry, novel zone entry, off‑hours movement) whenever new events are ingested.
The exact commands should be updated once the repository structure and framework choices have been finalized.

## Development Principles

- Start with deterministic simulated events before connecting real cameras or sensors.
- Implement baseline and anomaly rules before adding generative AI explanations.
- Keep detection results explainable and connected to evidence.
- Separate event ingestion, baseline evaluation, anomaly creation, and presentation.
- Treat camera and sensor data as sensitive information.
- Avoid claiming certainty when the system only has a confidence estimate.
- Preserve a human review step for consequential decisions.
- Keep the prototype focused on one environment before expanding to laboratories, factories, hospitals, or data centers.

## Security and Privacy

ODDEƎ may process camera footage, sensor data, operational records, and access-control events. Production deployments must define data retention, access control, encryption, audit logging, consent requirements, and appropriate handling of personally identifiable information.

The prototype should use synthetic or anonymized data. It must not be presented as a production security system until detection accuracy, false-positive behavior, privacy controls, and operational safety have been evaluated in realistic conditions.

## Roadmap

### Phase 1: Foundation

- Define the warehouse event schema.
- Create database migrations.
- Implement event ingestion.
- Build deterministic event replay.
- Add health checks and structured logging.

### Phase 2: Detection

- Define normal package locations.
- Define expected movement routes.
- Detect unexpected object movement.
- Correlate movement events with inventory transactions.
- Generate anomaly scores and explanations.

### Phase 3: Interface

- Build the warehouse floor-plan view.
- Add live event updates.
- Display anomaly markers.
- Create the evidence timeline.
- Add investigation status controls.

### Phase 4: Demonstration

- Create a repeatable demo scenario.
- Add seeded normal and anomalous events.
- Capture screenshots or a short product video.
- Prepare a concise explanation of the ODDEƎ concept.

### Phase 5: Expansion

- Add machine-state anomalies.
- Add access-control mismatches.
- Add laboratory workflow deviations.
- Add physical-digital correlation across more systems.
- Evaluate multimodal AI explanations.

## Contributing

Contributions should preserve the core principle of the project: detect and explain violations of expected physical behavior rather than merely identifying objects. Before submitting changes, run formatting, linting, tests, and any available integration checks.

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```

Frontend checks should be added when the frontend framework is selected.

## License

Add the project license here once the distribution model has been decided.

## Author

Created by Colin Stephenson.
