---
id: TASK-180
title: Restore release publishing and complete production deployment
status: In Progress
assignee:
  - '@codex'
created_date: '2026-10-04 07:30'
updated_date: '2026-10-04 08:45'
labels: []
dependencies: []
references:
  - 'https://hamgit.ir/hamdars/mahfel/-/jobs/7345767'
ordinal: 188000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Investigate registry authorization failure in job 7345767, correct release publishing, and run all four component release pipelines through Rastploy production deployments.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Registry upload failure has an evidence-backed cause and publishing fix
- [x] #2 All four component release pipelines build and publish versioned images
- [ ] #3 All four Rastploy deployment jobs succeed and live service readiness is verified
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect registry authorization and effective CI credentials. 2. Apply and validate the smallest publishing fix. 3. Push protected component releases and monitor full pipelines. 4. Resolve deployment failures and verify live readiness.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Job 7345767 authenticated but registry denied uploads to hamdars/mahfel-web. Runner and publisher credentials are identical. User selected existing aiedut namespace; upload initiation there returned 202. Changed CI and all four Compose targets in Mahfel 99a300a and Devops b08590c. Compose validation, six deployment-script tests, strict docs build and diff checks passed. Devops configuration rollout 7346123 succeeded. Full release pipelines: worker 2122085, media 2122086, web 2122087, API 2122088; monitoring through publish and deployment.

Production workspace screenshot traced to ServerApiRequestError 403. ADR 0019 provisioning step was absent from NextAuth callbacks. Added provisioning plus mandatory linked-token refresh on login, existing-session recovery, and fail-closed retry. Logo public-path fix included. Ten focused auth/proxy tests, ESLint and TypeScript pass. Turbopack build blocked by sandbox port binding; deployment/network check blocked by automatic approval service internal HTTP 404. Source remains uncommitted pending approval tooling recovery.

All four versioned app images were published. Corrected CD pipeline 2122323 passed API, media and web; worker retry 7347199 was submitted after isolated RabbitMQ account restoration and worker readiness recovery. Deployment acceptance remains open pending latest live verification. User requested commit/push: public-logo fix aeb64f2 and account provisioning fix dc95422 committed separately. Git/network approval tooling is responding again; release and live verification remain separate from this commit/push request.
<!-- SECTION:NOTES:END -->
