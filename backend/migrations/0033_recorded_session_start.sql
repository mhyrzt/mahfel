-- A capture preflight may outlive the API request. Its lease fences concurrent
-- starts and expires after a crash without changing the scheduled session.
ALTER TABLE mahfel.sessions
ADD COLUMN recording_start_token UUID NULL,
ADD COLUMN recording_start_expires_at TIMESTAMPTZ NULL,
ADD CONSTRAINT recording_start_lease_pair CHECK (
    (recording_start_token IS NULL) = (recording_start_expires_at IS NULL)
);

-- The original manifest remains immutable; the browser index is a derived,
-- versioned artifact accepted under the same finalization claim.
ALTER TABLE mahfel.recordings ADD COLUMN playback_index_key TEXT NULL;

-- Re-finalize earlier Ready bundles under new claims to derive browser media.
-- Original manifests, fragments and accepted exports remain untouched.
WITH legacy AS (
    UPDATE mahfel.recordings
    SET status = 'finalizing'
    WHERE status = 'ready' AND deleted_at IS NULL AND playback_index_key IS NULL
    RETURNING id, capture_generation
),

queued AS (
    INSERT INTO mahfel.background_jobs
    (
        id, job_type, schema_version, payload, idempotency_key, status,
        generation, attempt, created_at, updated_at
    )
    SELECT
        gen_random_uuid() AS id,
        'recordings.finalize' AS job_type,
        1 AS schema_version,
        jsonb_build_object('recording_id', id, 'capture_generation', capture_generation) AS payload,
        'recording.playback.backfill:' || id::TEXT || ':' || capture_generation::TEXT
            AS idempotency_key,
        'queued' AS status,
        1 AS generation,
        1 AS attempt,
        clock_timestamp() AS created_at,
        clock_timestamp() AS updated_at
    FROM legacy
    RETURNING id
)

INSERT INTO mahfel.job_outbox
(id, job_id, generation, attempt, available_at, created_at)
SELECT
    gen_random_uuid() AS id,
    id AS job_id,
    1 AS generation,
    1 AS attempt,
    clock_timestamp() AS available_at,
    clock_timestamp() AS created_at
FROM queued;
