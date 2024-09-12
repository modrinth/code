ALTER TABLE payouts_values ADD COLUMN date_available timestamptz NOT NULL DEFAULT now();
ALTER TABLE payouts_values ALTER COLUMN date_available DROP DEFAULT;