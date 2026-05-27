create table campaign_donations (
    id bigint primary key,
    raw_data jsonb not null,
    donated_at timestamptz not null,
    amount_usd numeric(96, 48),
    user_id bigint references users(id)
);
