# ADR 0020: Generic collaboration core with education as an integration

## Status

Accepted

## Context

Mahfel is described as an education-oriented realtime communication and collaboration platform, and
the roadmap has so far used classroom vocabulary: classrooms, teacher/student, attendance, waiting
rooms, teacher controls. That vocabulary leaked into the durable model only partially: the schema and
domain already model a generic `Room` with generic `RoomRole {Manager, Member}` and
`RoomVisibility`, while the live aggregate is still named `ClassroomSession` (table
`mahfel.classroom_sessions`) and M4 as planned talks about classroom lifecycles and teacher/student
attendance.

At the same time, external systems (for example `lms-core`, tracked by M2-09) are expected to own and
drive rooms through `mahfel-api` as service principals. If the core engine is built around one
vertical's words, every other consumer either inherits education semantics it does not want or the
team forks the engine later.

The engine primitives needed are not education-specific: a persistent space, a live occurrence of
that space, the people (or services) currently present, durable records of presence, capability-gated
moderation, admission control, speaker detection, and media-node placement.

## Decision

- The core domain, schema, and public API model **generic realtime collaboration**, not a learning
  management system. The durable vocabulary is:
  - `Room` — a persistent space.
  - `Session` — a live occurrence of a `Room` (renamed from `ClassroomSession`, table `sessions`).
  - `Participant` — a principal's stable identity within a live `Session`, surviving reconnect.
  - `Participation` record — durable join/leave intervals for a participant (the generic form of
    attendance).
  - Admission policy (`open` / `lobby`) — the generic form of a waiting room.
  - Capability-based authorization — moderation and session actions are gated by capabilities derived
    from global roles and room roles, not by a fixed vertical role enum.
- Education concepts are a **mapping**, not core types: teacher maps to a room manager/host
  capability, student to a participant, attendance to participation records, waiting room to lobby
  admission. Mapping guidance lives in documentation and in the consuming integration, not in
  `mahfel-domain`, the schema, or the `/v1` contract.
- No `teacher`, `student`, `classroom`, or `attendance` types, tables, or fields are introduced. The
  existing `ClassroomSession` / `classroom_sessions` naming is removed. Because the schema is not yet
  deployed beyond local development, migration `0002` is amended in place; a local database reset is
  required and the SQLx checksum consequence is documented.
- Service principals (M2-09) are first-class principals for ownership, membership, and capabilities,
  so an LMS or other external system can drive the generic engine without impersonating a user.
- Rooms and sessions do not carry a `kind`. Typed space kinds are deferred until a concrete
  behavioral requirement cannot be expressed with capabilities or policy.

## Consequences

- M4 is planned and implemented with generic names and capability checks. Education-facing copy may
  still say "classroom" in an education deployment, but the engine, database, and API stay generic.
- The domain rename touches `mahfel-domain`, `mahfel-application`, `mahfel-postgres`, the API
  transport, comments, and tests; migration `0002` and the migration test change with it.
- Moderation, admission, and session actions share one capability seam across REST and WebSocket, so
  new verticals add policy, not new role enums.
- M2-07's per-room `Manager`/`Member` model stays; the earlier `classroom:manage` style claim
  references in authorization code are cleaned up as part of the capability work.
- Future verticals (webinar, meeting, office hours) are expressed with the same primitives.

## Alternatives considered

- **Keep a classroom-native model.** Rejected: it couples the engine to one vertical and forces the
  LMS or any other consumer to inherit semantics it does not own.
- **Add a typed `RoomKind` (classroom, meeting, webinar).** Deferred: it multiplies schema and policy
  branches without a proven behavioral difference that capabilities cannot already express.
- **Add a fixed per-session role enum (Teacher/Student/Moderator).** Rejected: roles are already
  per-room state and the required distinctions are permission-shaped, so capabilities fit better and
  avoid a second role system.
