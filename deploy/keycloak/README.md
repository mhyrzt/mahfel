# Keycloak local identity provider

Keycloak is the single OIDC provider for Mahfel: the Compose `infrastructure` profile runs it for
local development and disposable integration tests, and production deployments point the same
standards-based OIDC client at their own Keycloak. There is no separate local-only provider.

The realm is `rastai` and the Compose issuer is:

```text
https://auth.mahfel.localhost/realms/rastai
```

Use the Caddy hostname for browser flows so the local trusted HTTPS certificate and issuer match.
The direct loopback port `http://127.0.0.1:8080` is useful for low-level debugging, but it is not the
configured issuer.

## Local run

The Compose service starts Keycloak in development mode with an in-memory database and imports
`rastai-realm.json` on every start:

```text
keycloak start-dev --import-realm --db=dev-mem --http-enabled=true \
  --hostname=https://auth.mahfel.localhost --hostname-backchannel-dynamic=true \
  --proxy-headers=xforwarded --health-enabled=true
```

Because the database is in-memory, the committed realm import is the source of truth for local
state: restarting the service always returns to this fixture. Administrative-console changes are
discarded on restart. That is intentional for reproducible local development; keep real changes in
`rastai-realm.json`.

No bootstrap administrator is configured. Mahfel does not manage the realm with an interactive
administrator account; it uses the `mahfel-admin` service account described below. To inspect the
realm through `https://auth.mahfel.localhost/admin`, add `KC_BOOTSTRAP_ADMIN_USERNAME` and
`KC_BOOTSTRAP_ADMIN_PASSWORD` to the Compose service for that session only.

## Development identities

All development users share the local-only password `password` and have stable subject IDs so
`(issuer, subject)` fixtures survive provider restarts.

| Username | Email | Realm role | Intended use |
| --- | --- | --- | --- |
| `shinji` | `shinji.ikari@mahfel.local` | `student` | learner flows |
| `asuka` | `asuka.langley@mahfel.local` | `student` | learner flows |
| `rei` | `rei.ayanami@mahfel.local` | `student` | learner flows |
| `misato` | `misato.katsuragi@mahfel.local` | `teacher` | instructor flows |
| `ritsuko` | `ritsuko.akagi@mahfel.local` | `moderator` | moderation/administration flows |
| `admin` | `admin@mahfel.local` | `admin` | administration flows |

These credentials are intentionally public local fixtures and must never be reused in production.

## Clients

| Client | Type | Purpose |
| --- | --- | --- |
| `mahfel-web` | public, PKCE | browser OIDC client (NextAuth); becomes a confidential client |
| `mahfel-api` | confidential (`mahfel-dev-secret`) | reserved; superseded by the `mahfel-web` browser flow (ADR 0019) |
| `mahfel-admin` | confidential service account (`mahfel-admin-dev-secret`) | Admin API access for `mahfel-api` through the client-credentials grant |

The API requests the `openid`, `profile`, `email`, and `roles` scopes, so Keycloak emits `sub`,
`preferred_username`, `name`, `email`, and `realm_access.roles` / `resource_access.*.roles`. Mahfel
authorization consumes its provider-neutral identity model and maps these claims only at the
authentication boundary.

## Managing the realm from `mahfel-api`

`mahfel-api` authenticates to the Keycloak Admin API with the client-credentials grant:

```text
POST https://auth.mahfel.localhost/realms/rastai/protocol/openid-connect/token
  grant_type=client_credentials
  client_id=mahfel-admin
  client_secret=<secret>
```

The `mahfel-admin` service account is granted the least-privilege `realm-management` client roles it
needs to provision and inspect users (`manage-users`, `view-users`, `query-users`, `view-realm`).
Interactive administrator credentials are never used by the application. The corresponding backend
settings live under `[keycloak]` (`base_url`, `realm`, `admin_client_id`, `admin_client_secret`).

The browser authenticates through `mahfel-web` with NextAuth and the API verifies the resulting
access token as a resource server (ADR 0019). No login route exists in the bootstrap yet, so this
provider setup is not a claim that browser login is already available.
