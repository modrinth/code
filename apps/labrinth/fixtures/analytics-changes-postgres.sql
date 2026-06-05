-- Dummy analytics data for exercising v3 analytics events and project download analytics.
-- IDs are listed as integers, followed by their equivalent base62 representation.

-- User 103587649610509 = 1XZwx9qL
INSERT INTO users (
	id, username, email, role, badges, balance, email_verified
)
VALUES (
	103587649610509, 'Analytics Admin', 'analytics-admin@modrinth.com',
	'admin', 0, 0, TRUE
)
ON CONFLICT (id) DO UPDATE SET
	role = EXCLUDED.role;

INSERT INTO sessions (
	id, session, user_id, expires, refresh_expires, ip, user_agent
)
VALUES (
	103587649610510, 'mra_analytics_admin', 103587649610509,
	'2030-01-01T00:00:00Z', '2030-01-01T00:00:00Z',
	'127.0.0.1', 'analytics fixture'
)
ON CONFLICT (session) DO UPDATE SET
	user_id = EXCLUDED.user_id,
	expires = EXCLUDED.expires,
	refresh_expires = EXCLUDED.refresh_expires;

-- Project 910000000000003 = 4AP3jpvKl
-- Team 910000000000001 = 4AP3jpvKj
-- Thread 910000000000004 = 4AP3jpvKm
INSERT INTO teams (id)
VALUES (910000000000001)
ON CONFLICT (id) DO NOTHING;

INSERT INTO team_members (
	id, team_id, user_id, role, permissions, accepted, payouts_split, ordering,
	organization_permissions, is_owner
)
VALUES (
	910000000000002, 910000000000001, 103587649610509, 'Owner',
	1023, TRUE, 100, 0, NULL, TRUE
)
ON CONFLICT (id) DO UPDATE SET
	permissions = EXCLUDED.permissions,
	accepted = EXCLUDED.accepted,
	is_owner = EXCLUDED.is_owner;

INSERT INTO mods (
	id, team_id, name, summary, downloads, slug, description, follows,
	license, status, requested_status, monetization_status,
	side_types_migration_review_status, components
)
VALUES (
	910000000000003, 910000000000001, 'Analytics Fixture Project',
	'Project used by analytics fixture data.', 0, 'analytics-fixture-project',
	'', 0, 'LicenseRef-All-Rights-Reserved', 'approved', 'approved',
	'monetized', 'reviewed', '{}'::jsonb
)
ON CONFLICT (id) DO UPDATE SET
	team_id = EXCLUDED.team_id,
	status = EXCLUDED.status,
	requested_status = EXCLUDED.requested_status,
	monetization_status = EXCLUDED.monetization_status;

INSERT INTO threads (id, thread_type, mod_id)
VALUES (910000000000004, 'project', 910000000000003)
ON CONFLICT (id) DO NOTHING;

-- Analytics events used to test /v3/analytics-event and Redis caching.
-- Event 910000000000101 = 4AP3jpvMR
-- Event 910000000000102 = 4AP3jpvMS
INSERT INTO analytics_events (id, meta, starts, ends)
VALUES
	(
		910000000000101,
		'{
			"title": "Downloads launch",
			"announcement_url": "https://modrinth.com/news/downloads-launch",
			"for_metric_kind": ["downloads"]
		}'::jsonb,
		'2026-05-13T00:00:00Z',
		'2026-05-14T00:00:00Z'
	),
	(
		910000000000102,
		'{
			"title": "Revenue promo",
			"announcement_url": "https://modrinth.com/news/revenue-promo",
			"for_metric_kind": ["revenue"]
		}'::jsonb,
		'2026-05-14T00:00:00Z',
		'2026-05-15T00:00:00Z'
	)
ON CONFLICT (id) DO UPDATE SET
	meta = EXCLUDED.meta,
	starts = EXCLUDED.starts,
	ends = EXCLUDED.ends;
