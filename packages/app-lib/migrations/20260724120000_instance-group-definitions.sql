ALTER TABLE instance_groups RENAME TO legacy_instance_group_memberships;

CREATE TABLE instance_groups (
	id TEXT NOT NULL,
	name TEXT NOT NULL,

	PRIMARY KEY (id),
	UNIQUE (name)
);

CREATE TABLE instance_group_memberships (
	instance_id TEXT NOT NULL,
	group_id TEXT NOT NULL,

	PRIMARY KEY (instance_id, group_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE,
	FOREIGN KEY (group_id) REFERENCES instance_groups(id) ON DELETE CASCADE
);

INSERT INTO instance_groups (id, name)
SELECT
	lower(hex(randomblob(16))),
	group_name
FROM legacy_instance_group_memberships
GROUP BY group_name;

INSERT INTO instance_group_memberships (instance_id, group_id)
SELECT
	legacy.instance_id,
	groups.id
FROM legacy_instance_group_memberships legacy
INNER JOIN instance_groups groups ON groups.name = legacy.group_name;

DROP TABLE legacy_instance_group_memberships;

CREATE INDEX instance_groups_name ON instance_groups(name);
CREATE INDEX instance_group_memberships_group_id
	ON instance_group_memberships(group_id);
