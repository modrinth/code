DROP INDEX IF EXISTS delphi_rule_effects_rule_id;

CREATE INDEX IF NOT EXISTS delphi_rule_effects_rule_id_detail_id
ON delphi_rule_effects (rule_id, detail_id DESC)
INCLUDE (revision, severity);
