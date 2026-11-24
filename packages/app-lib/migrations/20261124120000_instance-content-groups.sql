CREATE TABLE instance_content_groups (
	id TEXT NOT NULL,
	instance_id TEXT NOT NULL,
	content_type TEXT NOT NULL,
	name TEXT NOT NULL,
	display_order INTEGER NOT NULL DEFAULT 0,

	PRIMARY KEY (id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE,
	CHECK (length(trim(name)) > 0)
);

CREATE INDEX instance_content_groups_instance_type_order
	ON instance_content_groups(instance_id, content_type, display_order);

CREATE TABLE instance_content_group_memberships (
	instance_id TEXT NOT NULL,
	content_type TEXT NOT NULL,
	item_key TEXT NOT NULL,
	group_id TEXT NOT NULL,

	PRIMARY KEY (instance_id, content_type, item_key),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE,
	FOREIGN KEY (group_id) REFERENCES instance_content_groups(id) ON DELETE CASCADE
);

CREATE INDEX instance_content_group_memberships_group_id
	ON instance_content_group_memberships(group_id);
