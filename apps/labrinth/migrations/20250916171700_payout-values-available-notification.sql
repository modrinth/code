CREATE TABLE payouts_values_notifications (
	id SERIAL PRIMARY KEY,
	date_available TIMESTAMPTZ NOT NULL,
	user_id BIGINT NOT NULL REFERENCES users (id),
	notified BOOLEAN NOT NULL
);

CREATE UNIQUE INDEX payouts_values_notifications_date_available_user_id_idx ON payouts_values_notifications (
	date_available,
	user_id
);
