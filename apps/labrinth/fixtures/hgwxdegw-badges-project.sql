-- Fixture for user HGwXDEgw.
-- User 60829878552966 = HGwXDEgw
-- Team 930000000000001 = 4G5AdLiy1
-- Team member 930000000000002 = 4G5AdLiy2
-- Project 930000000000003 = 4G5AdLiy3
-- Thread 930000000000004 = 4G5AdLiy4
-- Pride donation 930000000000005 = 4G5AdLiy5

INSERT INTO users (
	id, username, email, role, badges, balance, email_verified
)
VALUES (
	60829878552966, 'fixture_hgwxdegw', 'admin@modrinth.invalid',
	'developer', 15, 0, TRUE
)
ON CONFLICT (id) DO UPDATE SET
	badges = users.badges | EXCLUDED.badges,
	email = COALESCE(users.email, EXCLUDED.email),
	email_verified = TRUE;

INSERT INTO teams (id)
VALUES (930000000000001)
ON CONFLICT (id) DO NOTHING;

INSERT INTO team_members (
	id, team_id, user_id, role, permissions, accepted, payouts_split, ordering,
	organization_permissions, is_owner
)
VALUES (
	930000000000002, 930000000000001, 60829878552966, 'Owner',
	1023, TRUE, 100, 0, NULL, TRUE
)
ON CONFLICT (id) DO UPDATE SET
	team_id = EXCLUDED.team_id,
	user_id = EXCLUDED.user_id,
	permissions = EXCLUDED.permissions,
	accepted = EXCLUDED.accepted,
	is_owner = EXCLUDED.is_owner;

INSERT INTO mods (
	id, team_id, name, summary, downloads, slug, description, follows,
	license, status, requested_status, monetization_status,
	side_types_migration_review_status, components
)
VALUES (
	930000000000003, 930000000000001, 'HGwXDEgw Million Download Fixture',
	'Project used to exercise badges and high download counts.', 1000000,
	'hgwxdegw-million-download-fixture', '', 0,
	'LicenseRef-All-Rights-Reserved', 'approved', 'approved',
	'monetized', 'reviewed', '{}'::jsonb
)
ON CONFLICT (id) DO UPDATE SET
	team_id = EXCLUDED.team_id,
	name = EXCLUDED.name,
	summary = EXCLUDED.summary,
	downloads = EXCLUDED.downloads,
	slug = EXCLUDED.slug,
	status = EXCLUDED.status,
	requested_status = EXCLUDED.requested_status,
	monetization_status = EXCLUDED.monetization_status,
	side_types_migration_review_status = EXCLUDED.side_types_migration_review_status,
	components = EXCLUDED.components;

INSERT INTO threads (id, thread_type, mod_id)
VALUES (930000000000004, 'project', 930000000000003)
ON CONFLICT (id) DO UPDATE SET
	thread_type = EXCLUDED.thread_type,
	mod_id = EXCLUDED.mod_id;

INSERT INTO campaign_donations (
	id, tiltify_event_id, raw_data, donated_at, amount_usd, user_id
)
VALUES (
	930000000000005, '00000000-0000-4000-8000-000000000005',
	'{"fixture": "hgwxdegw-badges-project"}'::jsonb,
	'2026-06-01T00:00:00Z', 5, 60829878552966
)
ON CONFLICT (id) DO UPDATE SET
	amount_usd = EXCLUDED.amount_usd,
	user_id = EXCLUDED.user_id;
