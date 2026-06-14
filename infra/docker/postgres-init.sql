-- Ferroway PostgreSQL initialization
-- Runs once when the container is first created

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- ─── Pipeline state schema ──────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS pipeline_runs (
    id          SERIAL PRIMARY KEY,
    run_id      UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    profile_id  VARCHAR(255) NOT NULL,
    started_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at    TIMESTAMPTZ,
    status      VARCHAR(50) NOT NULL DEFAULT 'running'
                CHECK (status IN ('running', 'completed', 'failed'))
);

CREATE TABLE IF NOT EXISTS pipeline_messages (
    id              BIGSERIAL PRIMARY KEY,
    run_id          UUID REFERENCES pipeline_runs(run_id),
    device_id       VARCHAR(255) NOT NULL,
    profile_id      VARCHAR(255) NOT NULL,
    channel_id      VARCHAR(255) NOT NULL,
    timestamp_nanos BIGINT NOT NULL,
    unit            VARCHAR(50),
    quality         VARCHAR(20) NOT NULL CHECK (quality IN ('good', 'degraded', 'fault', 'dropout')),
    source          VARCHAR(10) NOT NULL CHECK (source IN ('raw', 'derived')),
    data_type       VARCHAR(20) NOT NULL,
    value_numeric   DOUBLE PRECISION,
    received_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dead_letter_queue (
    id              BIGSERIAL PRIMARY KEY,
    message_json    JSONB NOT NULL,
    reason          TEXT NOT NULL,
    gate_condition  TEXT,
    received_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ─── Ferroway Forge / RAG schema ────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS document_chunks (
    id          BIGSERIAL PRIMARY KEY,
    doc_path    TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    content     TEXT NOT NULL,
    embedding   vector(1536),           -- OpenAI text-embedding-3-small dimensions
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (doc_path, chunk_index)
);

-- Index for fast similarity search
CREATE INDEX IF NOT EXISTS document_chunks_embedding_idx
    ON document_chunks
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100);

-- ─── Indexes ────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS pipeline_messages_channel_idx
    ON pipeline_messages (profile_id, channel_id);

CREATE INDEX IF NOT EXISTS pipeline_messages_timestamp_idx
    ON pipeline_messages (timestamp_nanos DESC);

CREATE INDEX IF NOT EXISTS pipeline_messages_quality_idx
    ON pipeline_messages (quality)
    WHERE quality IN ('fault', 'dropout');
