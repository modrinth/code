INSERT INTO teams (id) VALUES (RANDOM(1000000000,1000000000000));

INSERT INTO mods (
	id,
	team_id,
	name,
	summary,
	published,
	downloads,
	icon_url,
	updated,
	license_url,
	slug,
	description,
	follows,
	moderation_message,
	moderation_message_body,
	approved,
	license,
	status,
	requested_status,
	webhook_sent,
	color,
	queued,
	monetization_status,
	organization_id,
	raw_icon_url,
	side_types_migration_review_status
)
VALUES (
  	RANDOM(1000000000,1000000000000), (SELECT id FROM teams LIMIT 1), 'Test Mod', 'summary', NOW(), 0, '', NOW(), '', 'test-mod-2',
	'description.', 0, NULL, NULL, NOW(), '', 'unlisted', 'unlisted', FALSE, NULL,
	NOW(), 'demonetized', NULL, '', 'reviewed');

