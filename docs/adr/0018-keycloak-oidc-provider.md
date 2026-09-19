# ADR 0018: Use Keycloak as the single OIDC provider

## Status

Accepted

Partially superseded by [ADR 0019](0019-frontend-oidc-bff.md): the browser OIDC client and session
moved to `mahfel-web`, and `mahfel-api` verifies tokens as a resource server. Realm management
through the `mahfel-admin` service account is unchanged.

## Context

The bootstrap ran Dex for local development and disposable integration tests while production used
Keycloak. That split meant the authentication path was developed and verified against a provider the
project does not deploy, and it forced the OIDC client to tolerate two subtly different claim and
endpoint shapes. Mahfel also needs to provision and manage identities in the provider, which
previously implied holding interactive administrator credentials.

## Decision

- Keycloak is the single OIDC provider for local development, integration tests, and production.
- The Compose `infrastructure` profile runs Keycloak in development mode with an in-memory database
  and imports the committed realm `rastai` from `deploy/keycloak/rastai-realm.json` on every start.
  The realm import is the source of truth for local state.
- The OIDC client remains in `mahfel-api`: standards-based discovery, authorization code with PKCE,
  and the reserved callback `https://api.mahfel.localhost/v1/auth/oidc/callback`.
- `mahfel-api` manages the realm through the Keycloak Admin API using the client-credentials grant of
  a dedicated service account (`mahfel-admin`) with least-privilege `realm-management` roles. The
  application never uses an interactive administrator account.
- Production points the same provider-neutral OIDC configuration at its own Keycloak deployment;
  only the issuer URL and client secret differ.

## Consequences

One provider is validated end to end, and local fixtures are reproducible because the realm import
is committed and re-applied on every start. Administrative-console changes made locally are
discarded on restart by design. The Keycloak Admin API is vendor-specific and must stay behind an
adapter so it does not leak into application or domain code. The `mahfel-admin` service-account
secret is a deployment secret and must be overridden outside local development.

## Alternatives considered

Keeping Dex locally was rejected because it validates a provider that is not deployed and keeps two
claim shapes alive. TinyAuth would have added a third local-only provider. Provisioning through an
administrator username and password was rejected because it would require the application to hold
interactive administrator credentials.
