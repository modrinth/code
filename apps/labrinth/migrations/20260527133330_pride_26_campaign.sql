create table campaign_donations (
    id bigint primary key,
    raw_data jsonb not null,
    donated_at timestamptz not null,
    amount_usd numeric(96, 48),
    user_id bigint references users(id)
);
create index campaign_donations_user_amount_donated_at_idx
on campaign_donations (user_id, amount_usd, donated_at);
