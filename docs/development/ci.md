# Continuous integration

Mahfel uses GitLab CI with a root workflow, shared image jobs under `ci/`, and component jobs in
`backend/.gitlab-ci.yml` and `frontend/.gitlab-ci.yml`. Validation and build jobs are blocking;
image security scans are advisory and do not block subsequent stages when they fail.
The structure follows LMS, using Mahfel's pinned Rust and Bun tools and four runtime images.

## Pipeline triggers and scope

Merge requests and branch pushes validate changed components. A feature branch with an open merge
request uses the merge request pipeline instead of a duplicate push pipeline. Default
branch pushes also keep the existing Pages job even when only bookkeeping files change. Schedules and
branch pipelines started from the GitLab UI validate both components and protobuf. Component version
tags select the named component, plus shared docs and protobuf checks. Other tags do not create a
pipeline.

Backend changes, protobuf changes, and the Keycloak realm fixture select backend jobs. Frontend and
exported API contract changes select frontend jobs. Shared CI changes, the root pipeline, and
`.dockerignore` select both. Protobuf changes also run Buf formatting and linting. Documentation
changes run a strict MkDocs validation, including in merge requests. Docs Pages
publishing remains on the default branch. `docs-check` starts immediately and runs on every default
branch pipeline. Pages depends only on that job, so a successful docs check allows automatic
publishing without waiting for application jobs. Pages also performs its own strict build.

| Component | Checks |
| --- | --- |
| Backend | Rustfmt, Clippy with warnings denied, SQLFluff, nextest unit/binary tests, Docker-backed integration tests, doctests, three runtime image builds |
| Frontend | Prettier, ESLint, separate TypeScript validation, Vitest, Next.js production build, Storybook build, web runtime image build |
| Shared protocol | Buf formatting and lint |
| Images | Advisory Trivy HIGH/CRITICAL vulnerability checks for API, media, worker, and web |

Security scans retain their vulnerability reports and failing exit codes so findings remain
visible as job warnings. All four scan jobs use `allow_failure: true`, including release
pipelines, so findings and scanner failures do not block publishing or deployment. Backend
production runtimes remain on Debian 12 (`bookworm-slim`).

Backend compile jobs install Python and `python3-pip` alongside the native toolchain because
`mediasoup-sys` installs its Python build dependencies through `python3 -m pip`.
The unit-test job also installs the media runtime's GStreamer base/good/bad plugins so
recording readiness tests can check the required capture codecs.
Rust compile jobs limit Cargo to two build processes and disable dev/test debug information to
reduce memory use while compiling the AWS SDK and native media dependencies on the shared runner.

The `prepare` stage runs `cargo fetch --locked` and `bun install --frozen-lockfile` for the selected
components before application checks. These jobs populate lockfile/toolchain-keyed dependency
caches; subsequent jobs pull those caches without uploading competing copies. Installed Rust
tools use a separate cache, including Cargo's `.crates.toml` and `.crates2.json` installation
records alongside the binaries so repeated `cargo install` calls recognize existing tools.
Compiled `backend/target/` outputs are not shared between jobs:
mediasoup's Meson state contains absolute paths that break when restored into another runner
workspace. The tools cache uses a new key so older archives containing compiled targets are
not restored. Bun jobs still install from the frozen
lockfile into their own workspace using the prepared download cache. Docker builds keep their
separate BuildKit caches; they do not consume these job caches. System packages remain part of
each backend compile job's setup. Vitest runs with two workers to bound memory
and CPU use on shared runners. Trivy caches its databases. Nextest and
Vitest upload JUnit reports, including on failure. The CI nextest profile allows up to ten minutes
for slow infrastructure tests and writes `target/nextest/ci/junit.xml`.

The frontend test job uses Node 24.15.0 to execute Vitest and jsdom. It installs the pinned Bun
1.4.2 binary and still uses Bun for frozen dependency installation. GitHub's Bun release archive
must be reachable from the runner. Tests get a 15-second timeout for DOM-heavy interactions under
shared-runner load. The other frontend jobs use the Bun image.

## Docker runner requirements

Docker jobs require a Linux runner with Docker Buildx and access to the host daemon through
`/var/run/docker.sock`, matching the LMS socket-based runner. Images and the Trivy vulnerability
database must be reachable from that runner. No Docker-in-Docker service is started by the pipeline.

`backend-integration-test` builds the private `ci-integration` Dockerfile target. It contains the
native mediasoup toolchain and precompiled integration tests. The job launches that container with
host networking and the daemon socket, sets `MAHFEL_TESTCONTAINERS=1`, copies its JUnit report back,
and propagates the test exit code. Host networking is necessary because existing test harnesses
connect to disposable dependency ports through `127.0.0.1`. Testcontainers starts its own
PostgreSQL, Redis, RabbitMQ, RustFS, and Keycloak instances; shared databases are not configured.
The test container and its temporary image are removed when the job exits.

The runner must be restricted to trusted project code because jobs have Docker daemon access.
Do not enable these Docker jobs on a runner accepting arbitrary untrusted fork contributions.

## Image scanning and publishing

Build jobs produce compressed Docker archives. Separate scan jobs inspect those exact archives and
upload GitLab container-scanning reports using Trivy's GitLab report template. Any HIGH or CRITICAL
vulnerability fails the scan job with an allowed-failure warning; it does not fail the pipeline.
Unfixed vulnerabilities remain included in reports.

Mahfel has one release environment: Darkube production. Release tags use
`APP_NAME-vMAJOR.MINOR.PATCH`, where the app is `api`, `media`, `worker`, or `web`. Versions have
three numeric parts without leading zeroes; prerelease suffixes are not production release triggers.
Each component versions independently. A tag pipeline selects only that component's runtime image;
Rust tags still run the shared backend workspace checks. Every release runs `release-verify`, which
requires the tagged commit to be reachable from the default branch (`main`).

Publishing runs only for pushed, protected component release tags. The publish stage waits for all
preceding validation and builds to succeed and advisory scans to finish. It loads the same build
archive rather than rebuilding it. CI strips the app prefix from the Git release tag: `api-v0.1.0` publishes `api:v0.1.0`
and deploys Darkube image tag `v0.1.0`. The image repository identifies the app:

```text
registry.hamdocker.ir/hamdars/mahfel/api:v0.1.0
registry.hamdocker.ir/hamdars/mahfel/media:v0.1.0
registry.hamdocker.ir/hamdars/mahfel/worker:v0.1.0
registry.hamdocker.ir/hamdars/mahfel/web:v0.1.0
```

Main pushes, merge requests, other branches, schedules, and UI pipelines validate and scan without
publishing or deploying. Valid unprotected tags validate but cannot publish or deploy. There is no
production promotion bypass.

Protect `api-v*`, `media-v*`, `worker-v*`, and `web-v*` in GitLab and restrict who can create them.
Create an annotated release tag on a chosen main commit after validation:

```bash
git tag -a api-v0.1.0 <main-commit-sha> -m "Mahfel API v0.1.0"
git push origin api-v0.1.0
```

That tag releases API only. Use new version tags for subsequent releases; do not move, delete/reuse,
or overwrite an existing release. Configure registry tag immutability where available. The Git tag
records the release's commit SHA; runtime package versions are not rewritten by the pipeline.

Configure these GitLab CI/CD variables before publishing:

| Variable | Value |
| --- | --- |
| `CI_REGISTRY_USERNAME` | Registry account allowed to push Mahfel images |
| `CI_REGISTRY_PASSWORD` | Masked registry password or token, available to protected release tags |
| `MAHFEL_IMAGE_REGISTRY` | Defaults to `registry.hamdocker.ir` |
| `MAHFEL_IMAGE_PREFIX` | Defaults to `registry.hamdocker.ir/hamdars/mahfel`; override for another namespace |
| `NEXT_PUBLIC_MAHFEL_API_URL` | Public API URL used by validation builds; defaults to the local fixture URL |
| `MAHFEL_API_URL_PRODUCTION` | Public production API HTTPS URL for `web-v*` release builds |

Protect the release tag patterns so protected credentials are available. Web release builds require
`MAHFEL_API_URL_PRODUCTION`, so a missing setting cannot publish a web image compiled
with the local development URL. Other pipelines use `NEXT_PUBLIC_MAHFEL_API_URL`, which defaults to
the local URL. These are public values, never authentication tokens. The frontend's server-only OIDC configuration belongs
to runtime deployment secrets rather than CI build arguments.

Temporary Docker image tags and registry login state are removed after jobs; BuildKit keeps its
build cache. Image archives expire after one day; JUnit and scan reports expire after seven days. Retry an expired
publish job by rerunning the pipeline so its build archives are available again.

To publish images without deploying, supply `MAHFEL_SKIP_DEPLOY=true` to each tag pipeline:

```bash
git push -o ci.variable=MAHFEL_SKIP_DEPLOY=true origin api-v0.1.0
```

This omits all Darkube deployment jobs while retaining validation, advisory scans, and image
publishing. Push component tags individually so each tag receives its own release pipeline.

## Darkube deployment

The `deploy` stage runs after publishing and all quality gates. `backend-api-deploy`,
`backend-worker-deploy`, and `frontend-web-deploy` automatically deploy their component when its
protected version tag is pushed. They pass the exact published image tag to the same Darkube CLI used by LMS. Each app has a
separate resource group to serialize rollouts and a production environment entry. Deploy jobs do not
use `needs`, so they cannot bypass failed checks or publishing jobs.

`backend-media-deploy` is an optional manual job. Run it only after the public networking and media
drain requirements in [Darkube deployment](../deployment/darkube.md) have been qualified. An
ordinary CLI deployment does not implement graceful media draining or prove UDP connectivity.

Create four Docker-image apps on Darkube, configured to pull the corresponding image repository
above with registry credentials. Configure runtime secrets, private operations probes, and the
required production network topology on those apps. Database migrations must finish before rolling
API/worker versions that require them; configure that release operation before enabling the deploy
tokens. These CI jobs update existing apps, not create infrastructure or configure migrations.

Set the following GitLab variables for each component, using protected and masked deploy tokens:

| Component | App ID | Deploy token |
| --- | --- | --- |
| API | `MAHFEL_API_DARKUBE_APP_ID` | `MAHFEL_API_DARKUBE_DEPLOY_TOKEN` |
| Worker | `MAHFEL_WORKER_DARKUBE_APP_ID` | `MAHFEL_WORKER_DARKUBE_DEPLOY_TOKEN` |
| Web | `MAHFEL_WEB_DARKUBE_APP_ID` | `MAHFEL_WEB_DARKUBE_DEPLOY_TOKEN` |
| Media | `MAHFEL_MEDIA_DARKUBE_APP_ID` | `MAHFEL_MEDIA_DARKUBE_DEPLOY_TOKEN` |

Keep CI credentials available to these `production/*` environment entries. Missing app IDs or
tokens fail a deploy job before it invokes Darkube. Adding the jobs does not mean that a live
deployment or platform qualification has passed.

## Local validation

Use the existing component recipes for the same checks:

```bash
just backend fmt
just backend lint
just backend test
just frontend fmt
just frontend lint
just frontend typecheck
just frontend test
just frontend build
just frontend storybook-build
just proto fmt-check
just proto lint
just docs
```

GitLab's CI Lint validates the assembled pipeline and dependencies. Local YAML parsing cannot prove
runner image availability, registry access, Docker-backed acceptance, or a successful GitLab run.
