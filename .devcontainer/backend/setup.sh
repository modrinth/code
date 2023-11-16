#!/bin/bash
set -euo pipefail

cargo install --git https://github.com/launchbadge/sqlx sqlx-cli \
  --no-default-features --features postgres,rustls

(cd apps/labrinth && sqlx database setup)

psql "$DATABASE_URL" <<EOF
INSERT INTO loaders VALUES (0, 'placeholder_loader');
INSERT INTO loaders_project_types VALUES (0, 1); -- modloader id, supported type id
INSERT INTO categories VALUES (0, 'placeholder_category', 1); -- category id, category, project type id
EOF
