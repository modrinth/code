ALTER TABLE instance_groups RENAME TO legacy_instance_group_memberships;

CREATE TABLE instance_groups (
	id TEXT NOT NULL,
	name TEXT NOT NULL,
	display_order INTEGER NOT NULL DEFAULT 0,

	PRIMARY KEY (id)
);

CREATE TABLE instance_group_memberships (
	instance_id TEXT NOT NULL,
	group_id TEXT NOT NULL,

	PRIMARY KEY (instance_id, group_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE,
	FOREIGN KEY (group_id) REFERENCES instance_groups(id) ON DELETE CASCADE
);

INSERT INTO instance_groups (id, name, display_order)
SELECT
	lower(hex(randomblob(16))),
	group_name,
	ROW_NUMBER() OVER (ORDER BY group_name) - 1
FROM (
	SELECT DISTINCT group_name
	FROM legacy_instance_group_memberships
);

INSERT INTO instance_group_memberships (instance_id, group_id)
SELECT
	legacy.instance_id,
	groups.id
FROM legacy_instance_group_memberships legacy
INNER JOIN instance_groups groups ON groups.name = legacy.group_name;

UPDATE install_jobs
SET state = json_remove(
	json_set(
		state,
		'$.rollback.instance.group_ids',
		json(COALESCE((
			SELECT json_group_array(groups.id)
			FROM json_each(state, '$.rollback.instance.groups') legacy_groups
			INNER JOIN instance_groups groups
				ON groups.name = legacy_groups.value
		), '[]'))
	),
	'$.rollback.instance.groups'
)
WHERE json_type(state, '$.rollback.instance.groups') = 'array';

DROP TABLE legacy_instance_group_memberships;

CREATE INDEX instance_groups_name ON instance_groups(name);
CREATE INDEX instance_groups_display_order ON instance_groups(display_order);
CREATE INDEX instance_group_memberships_group_id
	ON instance_group_memberships(group_id);

INSERT OR IGNORE INTO instance_groups (id, name, display_order)
VALUES ('group:favorites', 'Favorites', -1);

UPDATE instance_groups
SET name = 'Favorites', display_order = -1
WHERE id = 'group:favorites';
