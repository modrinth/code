-- Enforces that there can only be one owner per team
CREATE UNIQUE INDEX idx_one_owner_per_team
ON team_members (team_id)
WHERE is_owner = TRUE;

-- Enforces one team_member per user/team
CREATE UNIQUE INDEX idx_unique_user_team
ON team_members (user_id, team_id);


