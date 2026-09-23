-- Development seed fixtures.
--
-- Applied explicitly with `mahfel-api seed` (or the `mahfel-seed` Compose
-- service), never by migrations and never during normal startup. The fixtures
-- are idempotent so the command can be re-run safely.
--
-- The development principal gives local development a stable authenticated
-- identity before real authentication (M2) exists. Nothing authenticates as
-- this user unless MAHFEL_API__DEVELOPMENT_PRINCIPAL_USER_ID is configured,
-- which is a local-only setting. Do not enable it in production.
-- Users share the actor id space, so the actor row is created first.
INSERT INTO mahfel.actors (id, kind, created_at, updated_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'user',
    '2026-01-01 00:00:00+00',
    '2026-01-01 00:00:00+00'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO mahfel.users (id, display_name, email, username, status, created_at, updated_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'Development Principal',
    'development-principal@mahfel.local',
    'development-principal',
    'active',
    '2026-01-01 00:00:00+00',
    '2026-01-01 00:00:00+00'
)
ON CONFLICT (id) DO NOTHING;
