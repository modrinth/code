CREATE TABLE users_subscriptions_affiliations (
	subscription_id BIGINT      NOT NULL REFERENCES users_subscriptions(id),
	affiliate_code  BIGINT      NOT NULL REFERENCES affiliate_codes(id),
	created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	deactivated_at  TIMESTAMPTZ,
	UNIQUE (subscription_id)
);

CREATE TABLE users_subscriptions_affiliations_payouts(
	id              BIGSERIAL          PRIMARY KEY,
	charge_id       BIGINT    NOT NULL REFERENCES charges(id),
	subscription_id BIGINT    NOT NULL REFERENCES users_subscriptions(id),
	affiliate_code  BIGINT    NOT NULL REFERENCES affiliate_codes(id),
	payout_value_id BIGSERIAL NOT NULL REFERENCES payouts_values(id),
	UNIQUE (charge_id),
	UNIQUE (payout_value_id)
);

ALTER TABLE payouts_values
ADD COLUMN affiliate_code_source BIGINT;
