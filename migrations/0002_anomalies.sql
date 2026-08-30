CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TYPE anomaly_severity AS ENUM (
  'low',
  'medium',
  'high',
  'critical'
);
CREATE TABLE anomalies (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  detected_at TIMESTAMPTZ NOT NULL,
  entity_id TEXT NOT NULL,
  source_event_id UUID NULL,
  severity anomaly_severity NOT NULL,
  reason TEXT NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'
);
CREATE INDEX anomalies_detected_at_idx ON anomalies (detected_at DESC);
CREATE INDEX anomalies_entity_id_idx ON anomalies (entity_id);
CREATE INDEX anomalies_severity_idx ON anomalies (severity);