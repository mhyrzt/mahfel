# Mahfel — Repository Instructions

Mahfel is a standalone education-oriented realtime communication and collaboration platform.

The repository contains four application components:

- `mahfel-web` — Next.js browser application
- `mahfel-api` — Rust control plane
- `mahfel-media` — Rust mediasoup media plane
- `mahfel-worker` — Rust background worker

Keep these deployment boundaries intact unless the user explicitly requests an architectural change.

## Instruction Scope

This file contains repository-wide rules.

Additional scoped instructions exist at:

- `backend/AGENTS.md`
- `frontend/AGENTS.md`

Before modifying anything under `backend/`, read and follow `backend/AGENTS.md`.

Before modifying anything under `frontend/`, read and follow `frontend/AGENTS.md`.

For work spanning both trees, read both files and apply each file to its own scope.

The more specific scoped instructions take precedence when they intentionally differ from this file.

## GitHub Issues Workflow

Use [GitHub issues](https://github.com/mhyrzt/mahfel/issues) and
[GitHub milestones](https://github.com/mhyrzt/mahfel/milestones) for task and project management.

At the beginning of repository work, search existing issues and read the relevant issue and
milestone before creating new work. Preserve unrelated issues and milestones.

Use `gh` from this repository or the GitHub API for `mhyrzt/mahfel`.
Read `gh <command> --help` before unfamiliar operations. Prefer structured API
request bodies for descriptions so Markdown and newlines remain intact.

Create an issue when work requires planning, decisions, or handoff notes; skip mechanical issue
creation for trivial work. Record the problem, scope, acceptance criteria, dependencies, and
validation evidence in the issue. Assign the appropriate milestone and verified GitHub assignees.

Use `status::To Do`, `status::In Progress`, and `status::Done` labels for task progress. Close done
issues after relevant validation and update their final summary. Archived issues remain closed
with `status::Archived`; archival does not imply acceptance criteria were completed.

Legacy task IDs are retained in migrated issue titles and original metadata. See
`docs/development/issue-tracking.md` for the migration mapping and conventions. GitHub is the
source of task state; do not recreate a local task database.

## Working Style

Before changing code:

1. Understand the requested behavior.
2. Inspect the smallest relevant implementation area.
3. Read the applicable architecture or ADR documents when the change touches a documented boundary.
4. Check the relevant GitHub issue or milestone.
5. Implement the smallest coherent change that solves the request.
6. Validate the changed surface.
7. Report what changed, what was validated, and any remaining risk.

Prefer working code over speculative abstractions.

Do not implement future roadmap functionality merely because it appears in `docs/development/roadmap.md`.

Do not expand the scope of a task without a concrete reason.

When requirements are ambiguous but an existing convention, ADR, test, or implementation clearly resolves the ambiguity, follow the repository evidence instead of inventing a new convention.

## Repository Architecture

Treat these as hard architectural boundaries unless explicitly changing them through an architectural decision.

### Browser traffic

Normal browser paths are:

```text
Browser ──HTTPS/REST/WebSocket──> mahfel-api
Browser ──WebRTC────────────────> mahfel-media
Browser ──presigned transfer────> S3-compatible storage
```

`mahfel-web` is not a general-purpose proxy for backend APIs, WebSockets, media, or large file bodies.

WebRTC signaling passes through `mahfel-api`.

Negotiated media does not pass through `mahfel-api` or Next.js.

### Runtime services

`mahfel-api` owns authoritative control-plane/application behavior.

`mahfel-media` owns realtime SFU/media resources and private media control operations.

`mahfel-worker` owns asynchronous/background jobs.

Do not turn domain capabilities into new deployable microservices without explicit architectural justification.

### State ownership

- PostgreSQL is authoritative durable application state.
- Redis is replaceable ephemeral coordination state.
- RabbitMQ transports jobs/events and is not business truth.
- S3-compatible storage owns artifact bytes.
- Media nodes own volatile live-session media resources only.

Never make Redis, RabbitMQ, container filesystems, or in-process memory the sole durable source of user-visible business state.

## Architecture Decisions

Accepted ADRs under `docs/adr/` are architectural constraints, not suggestions.

When a change intentionally contradicts an accepted ADR:

1. Do not silently implement the contradiction.
2. Surface the conflict.
3. If the user wants the architectural change, create a superseding ADR rather than rewriting the historical decision.

Minor documentation corrections that do not alter the decision are fine.

Architecture documentation should describe the implementation that actually exists. Do not document planned functionality as already implemented.

## Shared Contracts

`proto/` contains internal protocol contracts shared across backend components.

When changing protobuf:

- preserve compatibility unless a deliberate breaking change is requested;
- run Buf formatting and linting;
- update producers and consumers together when necessary;
- do not expose the private media gRPC contract directly to browsers.

Use:

```bash
just proto fmt-check
just proto lint
```

## Dependencies

Do not introduce a new dependency when the existing stack already provides a clear solution.

Before adding a production dependency, check:

- whether an existing dependency already solves the problem;
- maintenance status;
- security implications;
- runtime/build cost;
- whether it violates an existing ADR.

Avoid adding frameworks or infrastructure merely to reduce a small amount of local code.

## Security

Treat all external input as untrusted.

Never:

- hardcode production secrets;
- log credentials, tokens, cookies, authorization headers, private keys, or password material;
- expose operations ports publicly;
- expose internal media gRPC publicly;
- weaken authentication or authorization to make a test pass;
- invent cryptographic primitives.

Development credentials already committed as explicit local fixtures must remain clearly development-only.

Authorization should be deny-by-default.

## Observability

Preserve correlation across service boundaries where the infrastructure supports it.

Logs should be structured.

Metrics must use bounded-cardinality labels.

Never add identifiers such as these as Prometheus labels:

- user IDs
- room IDs
- participant IDs
- producer/consumer IDs
- email addresses
- arbitrary request values

High-cardinality identifiers belong in appropriately protected logs or traces.

Liveness should answer whether the process itself is alive.

Readiness should answer whether the service can currently perform useful work.

Do not turn normal dependency degradation into restart loops.

## Documentation

Update documentation when a change alters:

- architecture;
- public or internal contracts;
- development commands;
- configuration;
- deployment assumptions;
- operational behavior;
- documented invariants.

Prefer updating the nearest authoritative document instead of duplicating the same explanation in multiple places.

## Validation Strategy

Use the narrowest useful feedback loop while developing.

Do not repeatedly run every repository gate after tiny edits.

Before finalizing, run the relevant component checks.

For backend work, use the commands defined by `backend/AGENTS.md`.

For frontend work, use the commands defined by `frontend/AGENTS.md`.

For shared protobuf changes:

```bash
just proto fmt-check
just proto lint
```

For documentation changes:

```bash
just docs
```

For Compose/deployment changes:

```bash
docker compose config
```

For cross-component changes, validate every affected component.

If a full validation command cannot be run because required infrastructure or tooling is unavailable, run the strongest available subset and state exactly what was not validated.

## Change Discipline

Keep patches focused.

Do not:

- reformat unrelated files;
- rename unrelated APIs;
- rewrite working modules simply for stylistic preference;
- create empty speculative modules;
- duplicate an existing abstraction;
- leave dead compatibility layers without a reason.

When replacing an existing implementation, remove obsolete code when it is safe and within scope.

Prefer explicit migrations and compatibility transitions over hidden fallback behavior.

## Completion

A task is complete when:

- requested behavior exists;
- relevant tests or checks pass;
- architecture boundaries remain intact;
- documentation is updated when required;
- GitHub issue state and validation notes are finalized according to the GitHub workflow.

In the final response, summarize:

- what changed;
- important implementation decisions;
- validation performed;
- anything intentionally left unresolved.
