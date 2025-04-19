#!/bin/bash
set -euo pipefail

npm i -g corepack

corepack enable

COREPACK_ENABLE_DOWNLOAD_PROMPT=0 corepack install

cargo install --git https://github.com/launchbadge/sqlx sqlx-cli \
  --no-default-features --features sqlite,rustls

mkdir -p "$(dirname "${DATABASE_URL#sqlite://}")"

(cd packages/app-lib && cargo sqlx database setup)
