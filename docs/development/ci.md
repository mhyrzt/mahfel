# Continuous integration

Hosted continuous integration is currently disabled during the move to
[GitHub](https://github.com/mhyrzt/mahfel). The GitLab pipeline definitions and
GitLab-specific build, release, and deployment helpers have been removed.
No GitHub Actions replacement is configured yet.

## Local validation

Run the checks relevant to your change before submitting a pull request:

```bash
just frontend fmt
just frontend lint
just frontend typecheck
just frontend test
just backend fmt
just backend lint
just backend test
just backend build
just proto fmt-check
just proto lint
just docs
docker compose config
```

Frontend commands use Bun; backend commands use the pinned Rust toolchain.
Docker-backed integration tests require a working Docker daemon. See
[getting started](getting-started.md) for prerequisites and component instructions
for more focused validation commands.

Image publication and production deployment are currently manual. This repository
does not require the former private `devops` submodule for development or builds.
