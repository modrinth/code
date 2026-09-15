#!/usr/bin/env bash
# Bootstrap a clean Ubuntu VPS for Owyx site+API (NOT the VPN panel box).
# Run once as root or with sudo:  bash deploy/bootstrap-vps.sh
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "Re-run with sudo" >&2
  exit 1
fi

export DEBIAN_FRONTEND=noninteractive

apt-get update -y
apt-get install -y ca-certificates curl git nginx certbot python3-certbot-nginx ufw

# Swap (1–2G) — important on 2GB RAM boxes
if ! swapon --show | grep -q .; then
  fallocate -l 2G /swapfile || dd if=/dev/zero of=/swapfile bs=1M count=2048
  chmod 600 /swapfile
  mkswap /swapfile
  swapon /swapfile
  grep -q '/swapfile' /etc/fstab || echo '/swapfile none swap sw 0 0' >> /etc/fstab
fi

# Docker
if ! command -v docker >/dev/null 2>&1; then
  curl -fsSL https://get.docker.com | sh
  systemctl enable --now docker
fi

mkdir -p /var/www/certbot /opt/owyx
ufw allow OpenSSH
ufw allow 80/tcp
ufw allow 443/tcp
ufw --force enable || true

echo
echo "OK — next steps:"
echo "  1) DNS A: owyx.site, www.owyx.site, api.owyx.site → this VPS"
echo "  2) git clone https://github.com/ebluffy/Owyx.git /opt/owyx && cd /opt/owyx/owyxsite"
echo "  3) copy prepared .env (LAUNCHER_CLIENT_KEY, secrets, TURNSTILE_SECRET_KEY)"
echo "  4) bash deploy/vps-up.sh"
echo "  5) ln -sf \$PWD/deploy/nginx-owyx.site.conf /etc/nginx/sites-enabled/owyx-site.conf"
echo "  6) certbot --nginx -d owyx.site -d www.owyx.site -d api.owyx.site"
echo "  7) nginx -t && systemctl reload nginx"
echo "See deploy/OBT_PREP.md"
