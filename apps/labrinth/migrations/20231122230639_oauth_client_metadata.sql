-- Add migration script here
ALTER TABLE
    oauth_clients
ADD
    COLUMN url text NULL,
ADD
    COLUMN description text NULL;