CREATE TABLE organizations (
    id bigint PRIMARY KEY,
    title varchar(255) NOT NULL, -- also slug
    description text NOT NULL,
    created_at timestamp NOT NULL DEFAULT now(),
    updated_at timestamp NOT NULL DEFAULT now(),
    team_id bigint NOT NULL REFERENCES teams(id) ON UPDATE CASCADE,

    icon_url varchar(255) NULL,
    color integer NULL
);

ALTER TABLE mods ADD COLUMN organization_id bigint NULL REFERENCES organizations(id) ON DELETE SET NULL;

-- Organization permissions only apply to teams that are associated to an organization
-- If they do, 'permissions' is considered the fallback permissions for projects in the organization
ALTER TABLE team_members ADD COLUMN organization_permissions bigint NULL;
