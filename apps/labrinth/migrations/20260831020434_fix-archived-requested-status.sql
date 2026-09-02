UPDATE mods
SET requested_status = 'approved'
WHERE requested_status = 'archived';
