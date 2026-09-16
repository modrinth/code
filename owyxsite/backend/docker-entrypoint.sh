#!/bin/sh
set -eu
mkdir -p /app/uploads/avatars /app/uploads/skins /app/uploads/packs
# Host volume mounts often arrive as root:root — fix so USER owyx can write.
if [ "$(id -u)" = "0" ]; then
  chown -R owyx:nodejs /app/uploads 2>/dev/null || true
  exec su-exec owyx "$@"
fi
exec "$@"
