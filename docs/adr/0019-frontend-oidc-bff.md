# ADR 0019: Browser authentication via `mahfel-web` with Keycloak and a narrow BFF

## Status

Accepted

## Context

Keycloak is the single OIDC provider (ADR 0018) and is shared with the organization's other
platforms. The browser needs an authenticated session, and `mahfel-api` must remain authoritative
for users, identity linkage, and authorization. The earlier M2 plan assumed `mahfel-api` was itself
the OIDC client and owned the browser session through an authorization-code callback
(`/v1/auth/oidc/callback`). That would require the API to manage browser cookies, CSRF, and token
refresh, and it duplicates an auth stack the organization already operates.

The organization's existing web platform authenticates the browser with NextAuth against Keycloak
and forwards requests to its backend through a backend-for-frontend that attaches the access token.
Mahfel adopts the same shape so operators and developers have one mental model.

## Decision

- `mahfel-web` is the browser OIDC client. It uses NextAuth with the Keycloak provider: a
  confidential client, server-side code exchange, and `state` + PKCE (`S256`) + nonce. The session
  is an httpOnly cookie; tokens never reach browser JavaScript.
- `mahfel-web` exposes a deliberately narrow BFF: the NextAuth route handlers and
  `/api/backend/*`. The BFF forwards only allowlisted `/v1` paths to `mahfel-api` with
  `Authorization: Bearer <access token>`, strips cookies and hop-by-hop headers, bounds the body,
  and applies upstream timeouts. It is not a general proxy: WebSocket, media, and presigned S3
  traffic never pass through it.
- `mahfel-api` is a resource server. It verifies Keycloak access tokens (issuer, audience, JWKS with
  key rotation, clock skew), derives a `Principal` from the verified claims, and resolves or
  provisions the local user by `(issuer, subject)`. It never links identities by email and never
  runs an authorization-code login.
- Authorization stays deny-by-default in the API. Roles are read from the verified token and applied
  by local policy.
- WebSocket authentication reuses the same token verifier; the browser supplies the access token in
  the auth-first-message.
- This supersedes the API-owned authorization-code flow assumed by M2-03/M2-04/M2-05 and the
  reserved `mahfel-api` callback. The `mahfel-admin` service account remains for realm management.

## Consequences

Tokens stay server-side and the API stays the single authorization authority. The frontend becomes a
security-sensitive boundary and must be hardened: allowlisted upstream paths, no open proxy, strict
cookie flags, and secret management. CORS exposure shrinks because REST no longer originates from
the browser directly. The API must implement JWKS-based token verification and identity
provisioning, and the Keycloak client/realm fixtures must serve a confidential web client.

## Alternatives considered

Keeping `mahfel-api` as the OIDC client was rejected because it duplicates the organization's
existing NextAuth + Keycloak stack and forces the API to manage browser cookies and CSRF. Having the
browser call the API directly with a bearer token was rejected because tokens would reach
JavaScript, refresh handling would be duplicated, and CORS would widen.
