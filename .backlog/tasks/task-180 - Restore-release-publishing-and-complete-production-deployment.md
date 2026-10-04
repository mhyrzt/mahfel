---
id: TASK-180
title: Restore release publishing and complete production deployment
status: In Progress
assignee:
  - '@codex'
created_date: '2026-10-04 07:30'
updated_date: '2026-10-04 10:17'
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

User requested direct Keycloak sign-in from landing entry button and reported SessionExpired. Landing CTA now calls NextAuth signIn directly with workspace callback. Temporary AccountProvisioningUnavailable errors pass auth proxy so server session recovery can run; invalid refresh tokens remain blocked. Login displays readable safe error text instead of raw auth codes. Thirteen focused tests pass. Current production pipeline/session diagnosis remains unavailable because automatic approval review returns internal HTTP 404.

Live login incident root cause proven by PostgreSQL users_username_valid check failures at both provided session timestamps. Keycloak client, API audience, app_user_id mapper and mahfel-admin user-management roles audited correctly with reconciler. Added migration 0034 to allow email-shaped usernames on users and identities while retaining length, trim and whitespace constraints. Real disposable PostgreSQL migration/identity lifecycle regression passes (1 passed, 0 skipped); SQLFluff passes. Applied exact constraint correction transactionally to production with lock/statement timeouts; release migrator will record 0034 through SQLx normally.

API patch tag api-v1.0.3 at 162f4a4 triggers pipeline 2122668. Integration gate passed. Unit attempts failed before test execution on Rust distribution connection timeouts; latest retry 7348636 is compiling. Doc-test attempts failed downloading Python invoke from files.pythonhosted.org. Verified existing Devops repo.hmirror.ir Python mirror serves invoke on its own host, configured nonsecret project PIP_INDEX_URL and retry 7348649. Persisted mirror default in backend CI and documented it. No tests or security gates weakened.

User confirmed production sign-in works after the database constraint repair. Pipeline 2122668 integration gate passed; unit 7348636 and doc-test 7348649 continue. GitLab CI lint of the edited backend defaults is valid with no errors or warnings, strict docs build passes.
<!-- SECTION:NOTES:END -->
