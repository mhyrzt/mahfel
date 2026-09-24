CREATE TABLE mahfel.background_jobs (
    id UUID PRIMARY KEY,
    job_type TEXT NOT NULL CHECK (CHAR_LENGTH(job_type) BETWEEN 1 AND 128),
    schema_version SMALLINT NOT NULL CHECK (schema_version > 0),
    payload JSONB NOT NULL,
    idempotency_key TEXT NOT NULL CHECK (CHAR_LENGTH(idempotency_key) BETWEEN 1 AND 256),
    correlation_id TEXT NULL CHECK (CHAR_LENGTH(correlation_id) <= 128),
    traceparent TEXT NULL CHECK (CHAR_LENGTH(traceparent) <= 256),
    tracestate TEXT NULL CHECK (CHAR_LENGTH(tracestate) <= 512),
    status TEXT NOT NULL CHECK (status IN ('queued', 'running', 'succeeded', 'dead', 'cancelled')),
    generation INTEGER NOT NULL DEFAULT 1 CHECK (generation > 0),
    attempt SMALLINT NOT NULL DEFAULT 1 CHECK (attempt BETWEEN 1 AND 10),
    claim_token UUID NULL,
    lease_until TIMESTAMPTZ NULL,
    last_failure_code TEXT NULL CHECK (CHAR_LENGTH(last_failure_code) <= 128),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ NULL,
    UNIQUE (job_type, idempotency_key),
    CHECK ((claim_token IS NULL) = (lease_until IS NULL))
);

CREATE INDEX background_jobs_expired_claim_idx
ON mahfel.background_jobs (lease_until)
WHERE status = 'running';

CREATE TABLE mahfel.job_outbox (
    id UUID PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES mahfel.background_jobs (id),
    generation INTEGER NOT NULL CHECK (generation > 0),
    attempt SMALLINT NOT NULL CHECK (attempt BETWEEN 1 AND 10),
    available_at TIMESTAMPTZ NOT NULL,
    published_at TIMESTAMPTZ NULL,
    claim_token UUID NULL,
    lease_until TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL,
    UNIQUE (job_id, generation, attempt),
    CHECK ((claim_token IS NULL) = (lease_until IS NULL))
);

CREATE INDEX job_outbox_due_idx
ON mahfel.job_outbox (available_at, id)
WHERE published_at IS NULL;

CREATE INDEX job_outbox_published_idx
ON mahfel.job_outbox (published_at)
WHERE published_at IS NOT NULL;

CREATE TABLE mahfel.job_attempts (
    job_id UUID NOT NULL REFERENCES mahfel.background_jobs (id),
    generation INTEGER NOT NULL CHECK (generation > 0),
    attempt SMALLINT NOT NULL CHECK (attempt BETWEEN 1 AND 10),
    outcome TEXT NOT NULL CHECK (outcome IN ('retry', 'succeeded', 'dead')),
    failure_code TEXT NULL CHECK (CHAR_LENGTH(failure_code) <= 128),
    finished_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (job_id, generation, attempt)
);

CREATE TABLE mahfel.job_replays (
    job_id UUID NOT NULL REFERENCES mahfel.background_jobs (id),
    generation INTEGER NOT NULL CHECK (generation > 1),
    replayed_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (job_id, generation)
);
