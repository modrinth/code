UPDATE instances
SET
	submitted_time_played = CASE
		WHEN submitted_time_played < 0 THEN 0
		ELSE submitted_time_played
	END,
	recent_time_played = CASE
		WHEN recent_time_played < 0 THEN 0
		ELSE recent_time_played
	END
WHERE submitted_time_played < 0
	OR recent_time_played < 0;

CREATE TRIGGER instances_playtime_non_negative_insert
BEFORE INSERT ON instances
FOR EACH ROW
WHEN NEW.submitted_time_played < 0
	OR NEW.recent_time_played < 0
	OR typeof(NEW.submitted_time_played) != 'integer'
	OR typeof(NEW.recent_time_played) != 'integer'
BEGIN
	SELECT RAISE(ABORT, 'instance playtime must be a non-negative integer');
END;

CREATE TRIGGER instances_playtime_non_negative_update
BEFORE UPDATE OF submitted_time_played, recent_time_played ON instances
FOR EACH ROW
WHEN NEW.submitted_time_played < 0
	OR NEW.recent_time_played < 0
	OR typeof(NEW.submitted_time_played) != 'integer'
	OR typeof(NEW.recent_time_played) != 'integer'
BEGIN
	SELECT RAISE(ABORT, 'instance playtime must be a non-negative integer');
END;
