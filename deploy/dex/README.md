# Dex local identity provider

Dex is used only for local Docker Compose development and disposable integration tests. Production
deployments continue to use Keycloak through the same standard OIDC client configuration.

The Compose issuer is:

```text
https://auth.mahfel.localhost/dex
```

Use the Caddy hostname for browser flows so the local trusted HTTPS certificate and issuer match.
The direct loopback port `http://127.0.0.1:5556/dex` is useful for low-level debugging, but it is not
the configured issuer.

Development users all use the password `password`:

| Username | Email | Intended use |
| --- | --- | --- |
| `shinji` | `shinji.ikari@mahfel.local` | learner flows |
| `asuka` | `asuka.langley@mahfel.local` | learner flows |
| `rei` | `rei.ayanami@mahfel.local` | learner flows |
| `misato` | `misato.katsuragi@mahfel.local` | instructor flows |
| `ritsuko` | `ritsuko.akagi@mahfel.local` | administration/technical flows |
| `admin` | `admin@mahfel.local` | administration flows |

These credentials are intentionally public local fixtures and must never be reused in production.
Dex emits standard OIDC claims such as `sub`, `email`, `name`, and `preferred_username`; it does not
imitate Keycloak's `realm_access` or `resource_access` claims. Mahfel authorization must consume its
provider-neutral identity model and map provider-specific claims only at the authentication boundary.

The static client's redirect URI is the reserved API callback for M2. The bootstrap does not yet
implement a login or callback route, so this provider setup is not a claim that browser login is
already available.
