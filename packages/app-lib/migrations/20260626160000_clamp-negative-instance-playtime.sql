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
