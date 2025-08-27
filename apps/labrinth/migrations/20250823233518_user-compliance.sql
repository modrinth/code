CREATE TABLE users_compliance (
    id                     BIGSERIAL PRIMARY KEY,
    user_id                BIGINT NOT NULL REFERENCES users(id),

    requested              TIMESTAMP WITH TIME ZONE NOT NULL,
    signed                 TIMESTAMP WITH TIME ZONE,
    e_delivery_consented   BOOLEAN NOT NULL,
    tin_matched            BOOLEAN NOT NULL,
    last_checked           TIMESTAMP WITH TIME ZONE NOT NULL,

    external_request_id    VARCHAR NOT NULL,
    reference_id           VARCHAR NOT NULL,

    form_type              VARCHAR NOT NULL,

    UNIQUE (user_id)
);