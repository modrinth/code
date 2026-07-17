CREATE TABLE onboarding_checklist (
	id INTEGER NOT NULL CHECK (id = 0),
	has_created_instance INTEGER NOT NULL DEFAULT FALSE CHECK (has_created_instance IN (FALSE, TRUE)),
	has_logged_into_minecraft INTEGER NOT NULL DEFAULT FALSE CHECK (
		has_logged_into_minecraft IN (FALSE, TRUE)
	),
	has_logged_into_modrinth INTEGER NOT NULL DEFAULT FALSE CHECK (
		has_logged_into_modrinth IN (FALSE, TRUE)
	),
	show_checklist INTEGER NOT NULL DEFAULT TRUE CHECK (show_checklist IN (FALSE, TRUE)),
	PRIMARY KEY (id)
);

INSERT INTO onboarding_checklist
	(
		id,
		has_created_instance,
		has_logged_into_minecraft,
		has_logged_into_modrinth,
		show_checklist
	)
SELECT
	0,
	EXISTS (SELECT 1 FROM instances),
	EXISTS (SELECT 1 FROM minecraft_users),
	EXISTS (SELECT 1 FROM modrinth_users),
	NOT (
		EXISTS (SELECT 1 FROM instances)
		AND EXISTS (SELECT 1 FROM minecraft_users)
	);
