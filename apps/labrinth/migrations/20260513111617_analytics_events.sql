create table analytics_events (
    id bigint primary key,
    meta jsonb not null,
    starts timestamptz not null,
    ends timestamptz not null
);
