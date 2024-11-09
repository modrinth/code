-- Add migration script here
CREATE TABLE notifications (
    id bigint PRIMARY KEY,
    user_id bigint REFERENCES users NOT NULL,
    title varchar(255) NOT NULL,
    text varchar(2048) NOT NULL,
    link varchar(2048) NOT NULL,
    created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    read boolean DEFAULT FALSE NOT NULL
);

CREATE TABLE notifications_actions (
    id serial PRIMARY KEY,
    notification_id bigint REFERENCES notifications NOT NULL,
    title varchar(255) NOT NULL,
    action_route varchar(2048) NOT NULL,
    action_route_method varchar(32) NOT NULL
);