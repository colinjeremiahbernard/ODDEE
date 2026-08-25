CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE TYPE event_source AS ENUM (
  'camera',
  'sensor',
  'inventory',
  'access_control',
  'machine'
);
CREATE TYPE event_kind AS ENUM (
  'entered_zone',
  'exited_zone',
  'object_moved',
  'object_missing',
  'state_changed',
  'access_granted',
  'access_denied',
  'transaction_recorded'
);
CREATE TYPE anomaly_status AS ENUM (
  'new',
  'investigating',
  'resolved',
  'dismissed'
);
CREATE TABLE physical_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  occurred_at TIMESTAMPTZ NOT NULL,
  source event_source NOT NULL,
  entity_id TEXT NOT NULL,
  kind event_kind NOT NULL,
  zone TEXT NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE anomalies (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  severity TEXT NOT NULL,
  score REAL NOT NULL CHECK (
    score >= 0
    AND score <= 1
  ),
  title TEXT NOT NULL,
  explanation TEXT NOT NULL,
  status anomaly_status NOT NULL DEFAULT 'new',
  related_event_ids UUID [] NOT NULL DEFAULT '{}'
);
CREATE INDEX physical_events_occurred_at_idx ON physical_events (occurred_at DESC);
CREATE INDEX physical_events_entity_id_idx ON physical_events (entity_id);
CREATE INDEX anomalies_detected_at_idx ON anomalies (detected_at DESC);