#!/usr/bin/env bash

set -euo pipefail

source .env

psql "$DATABASE_URL" -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = current_database() AND pid <> pg_backend_pid();"
cargo sqlx database reset
../../scripts/seed-db.sh
