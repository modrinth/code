#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
LABRINTH_DIR="$REPO_ROOT/apps/labrinth"

if [[ -f "$LABRINTH_DIR/.env" ]]; then
	source "$LABRINTH_DIR/.env"
fi
if [[ -f "$LABRINTH_DIR/.env.local" ]]; then
	source "$LABRINTH_DIR/.env.local"
fi

: "${DATABASE_URL:?DATABASE_URL must be configured}"
: "${MINECRAFT_META_DATABASE_URL:?MINECRAFT_META_DATABASE_URL must be configured}"

psql -X --set=ON_ERROR_STOP=1 "$MINECRAFT_META_DATABASE_URL" \
	--file="$LABRINTH_DIR/fixtures/minecraft-meta-seed-data-202508052143.sql"
psql -X --set=ON_ERROR_STOP=1 "$DATABASE_URL" \
	--file="$LABRINTH_DIR/fixtures/labrinth-seed-data-202508052143.sql"
