# ADR 0022: Service principals authenticate with Mahfel-issued API keys

## Status

Accepted

## Context

External systems (for example `lms-core`, tracked by M2-09) must create, populate, and manage rooms
through `mahfel-api`. The identity model was user-only: the principal carried a single user id, and
`mahfel.rooms.created_by` and `mahfel.room_memberships.user_id` referenced `mahfel.users`. A service
therefore had no way to own a room or hold a membership without impersonating a human user, which
would corrupt ownership, audit, and authorization.

The core engine models generic collaboration (ADR 0020), and service principals are first-class
principals for ownership, membership, and capabilities. This record fixes the authentication
mechanism and the durable identity model.

## Decision

- **Unified actor model.** Users and service principals share one PostgreSQL id space in
  `mahfel.actors (id, kind)` with `kind IN ('user', 'service')`. `mahfel.users.id` and
  `mahfel.service_principals.id` each reference `mahfel.actors(id) ON DELETE CASCADE`.
  `mahfel.rooms.created_by` and `mahfel.room_memberships.actor_id` reference `actors(id)`. A room has
  one owner column and one membership column regardless of whether the owner is a person or a
  machine. Domain identifier `ActorId` replaces `UserId`.
- **Mahfel-issued API keys.** A service principal authenticates with an opaque bearer key of the form
  `mhf_` + 64 lowercase hex characters. Only the SHA-256 hex digest is stored
  (`mahfel.service_api_keys.key_hash`); the raw key is returned exactly once at creation or rotation
  and is never logged or persisted. Keys are independent of Keycloak: the API does not run a
  client-credentials login on behalf of a service and does not require a Keycloak service account.
- **Principal carries kind and actor id.** The authenticated `Principal` has `actor_id: ActorId` and
  `kind: ActorKind`. User tokens continue to use the `app_user_id` claim and default to
  `kind = user`. An optional `app_principal_kind` claim (`user` or `service`) is parsed for future
  use; an absent or unknown value is treated as `user`, so a malformed claim can never widen access
  to a machine actor.
- **Full actor parity, deny-by-default.** A service principal can create and own rooms and hold
  `manager` or `member` memberships exactly like a user, and is governed by the same
  capability/policy model. Service principals carry no global roles by default, so they can create
  and own rooms but cannot `ManageSystem` or globally `ModerateRoom`. Nothing in the model treats a
  service as an impersonated user.
- **Provisioning, rotation, revocation.** Administrators manage principals under
  `/v1/admin/service-principals`: `POST` creates (returns the raw key once), `GET` lists, `POST
  /{id}/rotate` revokes the active keys and issues a new raw key (returned once, refused for a
  revoked principal), and `DELETE /{id}` revokes the principal and its active keys. Every endpoint
  requires `Action::ManageSystem`. Revocation is terminal: subsequent requests with any key that
  belonged to the principal are rejected with `401`.
- **Local development provisioning.** The local Keycloak realm fixture seeds a user named `admin`
  with the `admin` realm role (`deploy/keycloak/README.md`). Bootstrapping a service principal
  locally is: log in through `mahfel-web` as that administrator, let provisioning link the local
  user, then call `POST /v1/admin/service-principals` with the resulting access token. There is no
  separate first-admin secret; the realm fixture is the bootstrap.

## Consequences

- New durable tables (`mahfel.actors`, `mahfel.service_principals`, `mahfel.service_api_keys`) and
  migration `0005_actors.sql`, which backfills one user actor per existing user and repoints room
  ownership and membership at actors.
- `mahfel-api` owns service principal records and API keys. `mahfel-media` and `mahfel-worker` do not
  own them and never see raw keys.
- API keys are bearer credentials with no scopes or expiry in this iteration; rotation is the
  operator's revocation tool for a single key, and revocation for the whole principal. Operators
  must store raw keys in a secret manager because they cannot be recovered.
- Authorization remains deny-by-default and transport-agnostic: the API-key path produces the same
  `Principal` type as the JWT path, so downstream use cases and policy do not branch on the
  credential type.

## Alternatives considered

- **Keycloak client-credentials / service-account tokens.** Rejected for now: it couples external
  systems to the organization's Keycloak realm and client administration, and requires provisioning
  a Keycloak client per integration. Mahfel-issued keys keep the control plane self-contained and
  make rotation and revocation a first-party operation. The principal `kind` seam keeps a future
  Keycloak-issued service token possible without changing the domain model.
- **A separate `service_actors` table.** Rejected: two id spaces force every owner/membership column
  to carry a discriminator and every query to union two tables. One actor table with a `kind`
  discriminator keeps referential integrity and the existing room model intact.
- **Giving services global roles or a service-specific role enum.** Rejected: service principals
  need ownership and room-scoped management, not system administration. Global roles stay with
  human administrators and the capability model handles room-scoped actions.
- **Storing raw keys or reversible ciphertext.** Rejected: a stolen database must not yield usable
  credentials. SHA-256 digests are sufficient for high-entropy opaque keys and match the
  "returned once" contract.
