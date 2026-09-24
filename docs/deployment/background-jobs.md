# Background jobs

The API writes `mahfel.background_jobs` and `mahfel.job_outbox` in the same PostgreSQL
transaction as a business change. `mahfel-worker` leases due outbox rows and publishes them to
RabbitMQ using mandatory routing and publisher confirms. The API can commit while RabbitMQ is
unavailable; the worker publishes pending rows after it reconnects.

## Job contract

The JSON envelope contains `job_id`, `job_type`, `version`, `created_at`, optional
`correlation_id`, `idempotency_key`, `generation`, `attempt`, and `payload`. Optional
`traceparent` and `tracestate` are also carried as AMQP headers. Version 1 uses the direct
exchange `mahfel.jobs.v1`. The first routing key is
`files.cleanup_deleted_attachment.v1`, bound to `mahfel.jobs.files.cleanup.v1`.
Payloads contain identifiers, not secrets or artifact bytes. Consumers accept additive fields
within a version; incompatible changes need a new version and a period of dual reading.

The worker acknowledges only after a durable effect or scheduled retry is recorded. Duplicate
deliveries are expected. PostgreSQL claims use a two-minute lease renewed every 30 seconds, and
the handler must be repeatable. File cleanup is repeatable because deleting an absent S3 object
or metadata row succeeds. Later handlers that change PostgreSQL data must commit their effect
and completion marker together. External providers need their own idempotency key.

## Retry and dead letters

The first attempt runs immediately. Retryable failures wait 30, 60, 120, 240, 480, then 600
seconds for each remaining retry, for ten total attempts. PostgreSQL `available_at` schedules
these attempts; no RabbitMQ delay plugin is required. Invalid payloads, unsupported versions,
and other permanent failures go straight to the DLQ. The source queue uses an at-least-once
dead-letter policy with `reject-publish` overflow. The DLQ keeps messages for seven days;
PostgreSQL retains dead job state and attempt history until an operator resolves it.

In local Minikube, `just minikube` applies the broker policy after RabbitMQ starts. Other
deployments must apply the equivalent quorum-queue policy before starting workers:

```bash
rabbitmqctl set_policy mahfel-job-dlx '^mahfel\.jobs\.files\.cleanup\.v1$' \
  '{"dead-letter-strategy":"at-least-once","overflow":"reject-publish","dead-letter-exchange":"mahfel.jobs.dead.v1","dead-letter-routing-key":"files.cleanup_deleted_attachment.v1","max-length-bytes":104857600}' \
  --apply-to quorum_queues
```

The dead-letter queue is `mahfel.jobs.files.cleanup.v1.dead`. Inspect it in the private RabbitMQ
management UI, and inspect the authoritative failure history with:

```bash
mahfel-worker jobs inspect JOB_UUID
```

After fixing the cause, replay one dead job with:

```bash
mahfel-worker jobs replay JOB_UUID
```

Replay retains the job ID and idempotency key, increments its generation, and records another
attempt. PostgreSQL records each replay generation and timestamp in `mahfel.job_replays`.
The old DLQ copy remains for investigation until its seven-day expiry. Do not publish
the raw DLQ body directly: it has an old generation and is intentionally ignored by the worker.

## Worker capacity and shutdown

`backend/config/default.toml` sets `worker.concurrency = 4`, `worker.prefetch = 4`, and
`worker.drain_timeout_seconds = 30`. Set a positive concurrency and prefetch for each worker;
prefetch controls unacknowledged RabbitMQ deliveries while concurrency limits tasks running in
the process. On shutdown, the worker stops taking deliveries and allows active tasks to finish
for the configured drain period. A job left without a completed PostgreSQL claim is recovered
after its two-minute lease expires.

## Monitoring

`mahfel-worker:9103/metrics` exports `mahfel_job_outbox_pending`,
`mahfel_job_outbox_oldest_seconds`, `mahfel_job_publishes_total{outcome}`,
`mahfel_jobs_active`, and `mahfel_job_outcomes_total{outcome}`. Labels contain only bounded
outcome values. Logs carry job IDs for investigation. Worker liveness stays available during a
broker or database outage; readiness reports whether both dependencies can currently support
job work.
