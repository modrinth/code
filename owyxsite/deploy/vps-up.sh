#!/usr/bin/env bash
# Bring up the Owyx site stack on a VPS (or Cloud VM) next to an existing
# host nginx / C³ CELERITY panel. Does not touch panel.owyx.site.
#
# Usage (from repo root or owyxsite/):
#   bash owyxsite/deploy/vps-up.sh           # prod overlay: loopback binds
#   bash owyxsite/deploy/vps-up.sh --dev     # default compose (all ports)
#
# Idempotent. Does not commit .env. Never binds 443 — host nginx owns TLS.
#
# Order: write/load .env → --dev vs prod Turnstile/NODE_ENV → source ports →
# audit → compose up → health. Do not audit or print "ready" on stale defaults.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

DEV_MODE=0
if [[ "${1:-}" == "--dev" ]]; then
  DEV_MODE=1
fi

docker_cmd() {
  if docker info >/dev/null 2>&1; then
    docker "$@"
  elif command -v sudo >/dev/null 2>&1 && sudo docker info >/dev/null 2>&1; then
    sudo docker "$@"
  else
    echo "error: docker is not running (need docker + compose plugin)" >&2
    exit 1
  fi
}

need_bin() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: missing command: $1" >&2
    exit 1
  fi
}

# 32 bytes → 64 hex chars. Fail (do not swallow) if urandom/od is broken.
rand() {
  local hex
  hex="$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')"
  if [[ "${#hex}" -lt 32 ]]; then
    echo "error: failed to read /dev/urandom for a secret" >&2
    exit 1
  fi
  printf '%s' "${hex}"
}

# Replace KEY=... in .env (or append). Values must be sed-safe (hex/tokens/URLs).
set_env_key() {
  local key="$1" val="$2"
  if grep -qE "^${key}=" .env; then
    sed -i "s|^${key}=.*|${key}=${val}|" .env
  else
    printf '%s=%s\n' "${key}" "${val}" >> .env
  fi
}

env_looks_placeholder() {
  local val="${1:-}"
  [[ -z "${val}" ]] && return 0
  case "${val}" in
    *change_me*|*generate_random*|*your_secure*|*your_admin*)
      return 0
      ;;
  esac
  return 1
}

load_env_file() {
  if [[ ! -f .env ]]; then
    echo "error: .env is missing" >&2
    exit 1
  fi
  set -a
  # shellcheck disable=SC1091
  . ./.env
  set +a
  FRONTEND_PORT="${FRONTEND_PORT:-3000}"
  API_PORT="${API_PORT:-3001}"
  POSTGRES_PORT="${POSTGRES_PORT:-5432}"
  ADMINER_PORT="${ADMINER_PORT:-8080}"
  FRONTEND_BIND="${FRONTEND_BIND:-}"
  API_BIND="${API_BIND:-}"
  POSTGRES_BIND="${POSTGRES_BIND:-}"
}

assert_secrets_are_real() {
  local key val
  for key in DB_PASSWORD JWT_SECRET SESSION_SECRET; do
    val="${!key:-}"
    if env_looks_placeholder "${val}"; then
      echo "error: ${key} is empty or still a .env.example placeholder" >&2
      echo "    generate a real value and re-run (script fails closed)." >&2
      exit 1
    fi
  done
}

apply_turnstile_policy() {
  if [[ "${DEV_MODE}" -eq 1 ]]; then
    # SKIP is honored only when NODE_ENV is not production (see turnstile.js).
    set_env_key NODE_ENV development
    return 0
  fi

  set_env_key TURNSTILE_SKIP false
  local secret="${TURNSTILE_SECRET_KEY:-${TURNSTILE_SECRET:-}}"
  if [[ -z "${secret}" ]]; then
    echo "error: production compose requires TURNSTILE_SECRET_KEY in .env" >&2
    echo "    TURNSTILE_SKIP is ignored when NODE_ENV=production — login/register fail-close." >&2
    echo "    Set TURNSTILE_SECRET_KEY (Cloudflare), or re-run with --dev for local skip." >&2
    exit 1
  fi
}

# Prod path is loopback + host nginx. .env.example ships 0.0.0.0; ${VAR:-127.0.0.1}
# does not override an already-set 0.0.0.0 after load_env_file. Always rewrite.
apply_prod_loopback_binds() {
  set_env_key FRONTEND_BIND "127.0.0.1"
  set_env_key API_BIND "127.0.0.1"
  set_env_key POSTGRES_BIND "127.0.0.1"
  set_env_key TRUST_PROXY_HOPS "${TRUST_PROXY_HOPS:-1}"
  FRONTEND_BIND=127.0.0.1
  API_BIND=127.0.0.1
  POSTGRES_BIND=127.0.0.1
  export FRONTEND_BIND API_BIND POSTGRES_BIND
  export TRUST_PROXY_HOPS="${TRUST_PROXY_HOPS:-1}"
}

echo "==> Owyx VPS / local compose up"
echo "    root: ${ROOT}"
need_bin curl
if ! command -v docker >/dev/null 2>&1; then
  echo "error: docker not installed" >&2
  exit 1
fi
if ! docker_cmd compose version >/dev/null 2>&1; then
  echo "error: docker compose plugin missing" >&2
  exit 1
fi

if command -v nginx >/dev/null 2>&1; then
  echo "    nginx: $(nginx -v 2>&1 || true) (host TLS — we do not bind :443)"
else
  echo "    nginx: not on PATH (optional; host panel/TLS is out of this script)"
fi

# --- .env first (compose and health both read these ports) ---
if [[ ! -f .env ]]; then
  echo "==> writing .env from .env.example (random secrets — not committed)"
  if [[ ! -f .env.example ]]; then
    echo "error: missing .env.example" >&2
    exit 1
  fi
  cp .env.example .env
  set_env_key DB_PASSWORD "$(rand)"
  set_env_key JWT_SECRET "$(rand)"
  set_env_key SESSION_SECRET "$(rand)"
  set_env_key DIRECTUS_KEY "$(rand)"
  set_env_key DIRECTUS_SECRET "$(rand)"
  if [[ "${DEV_MODE}" -eq 1 ]]; then
    set_env_key NODE_ENV development
    set_env_key TURNSTILE_SKIP true
    set_env_key FRONTEND_URL "http://localhost:3000"
    set_env_key PUBLIC_URL "http://localhost:3000"
    set_env_key CORS_ORIGIN "http://localhost:3000"
    set_env_key NEXT_PUBLIC_API_URL "http://localhost:3000/api"
  else
    set_env_key NODE_ENV production
    set_env_key TURNSTILE_SKIP false
    apply_prod_loopback_binds
  fi
  echo "    created ${ROOT}/.env (keep it off git)"
else
  echo "==> using existing .env"
fi

if [[ "${DEV_MODE}" -eq 0 ]]; then
  apply_prod_loopback_binds
fi
load_env_file
export FRONTEND_PORT API_PORT POSTGRES_PORT ADMINER_PORT

assert_secrets_are_real
apply_turnstile_policy
# Policy may have rewritten NODE_ENV / TURNSTILE_*; re-source so compose + health match the file.
if [[ "${DEV_MODE}" -eq 0 ]]; then
  apply_prod_loopback_binds
fi
load_env_file
export FRONTEND_PORT API_PORT POSTGRES_PORT ADMINER_PORT

port_listeners() {
  local port="$1"
  if command -v ss >/dev/null 2>&1; then
    ss -tlnp 2>/dev/null | awk -v p=":${port}" '$4 ~ p"$" || $4 ~ p" "' || true
  elif command -v lsof >/dev/null 2>&1; then
    lsof -nP -iTCP:"${port}" -sTCP:LISTEN 2>/dev/null || true
  else
    echo "(no ss/lsof — skipped)"
  fi
}

is_owyx_container_port() {
  local port="$1"
  docker_cmd ps --format '{{.Names}} {{.Ports}}' 2>/dev/null | grep -E "owyx-" | grep -q ":${port}->" || return 1
}

suggest_alt() {
  local name="$1" current="$2"
  echo "    set ${name}=$((current + 10)) (or another free port) in .env and re-run"
}

echo "==> port audit (from .env: frontend=${FRONTEND_PORT} api=${API_PORT} postgres=${POSTGRES_PORT})"
PROBLEMS=0
audit_port() {
  local label="$1" port="$2" envname="$3"
  local listeners
  listeners="$(port_listeners "${port}")"
  if [[ -z "${listeners}" || "${listeners}" == "(no ss/lsof — skipped)" ]]; then
    echo "    ${label} :${port}  free"
    return 0
  fi
  if is_owyx_container_port "${port}"; then
    echo "    ${label} :${port}  our owyx container — will recreate"
    return 0
  fi
  echo "    ${label} :${port}  BUSY (not an owyx-* container):"
  echo "${listeners}" | sed 's/^/      /'
  suggest_alt "${envname}" "${port}"
  PROBLEMS=1
}

audit_port "frontend" "${FRONTEND_PORT}" "FRONTEND_PORT"
audit_port "api" "${API_PORT}" "API_PORT"
audit_port "postgres" "${POSTGRES_PORT}" "POSTGRES_PORT"
if [[ "${DEV_MODE}" -eq 1 ]]; then
  audit_port "adminer" "${ADMINER_PORT}" "ADMINER_PORT"
fi

if [[ "${PROBLEMS}" -ne 0 ]]; then
  echo "error: one or more ports are taken by something else. Fix .env and retry." >&2
  echo "    Host nginx on :80/:443 is expected and OK — we only bind app ports." >&2
  echo "    panel.owyx.site (C³ CELERITY) must keep its own server block — do not steal :443." >&2
  exit 1
fi

if docker_cmd ps --format '{{.Names}}' 2>/dev/null | grep -qE '^owyx-(frontend|backend|postgres|adminer)$'; then
  echo "==> stopping previous owyx compose stack"
  if [[ "${DEV_MODE}" -eq 1 ]]; then
    docker_cmd compose down --remove-orphans || true
  else
    docker_cmd compose -f docker-compose.yml -f docker-compose.prod.yml down --remove-orphans || true
  fi
fi

COMPOSE=(compose)
if [[ "${DEV_MODE}" -eq 0 ]]; then
  COMPOSE+=( -f docker-compose.yml -f docker-compose.prod.yml )
fi

echo "==> docker compose up -d --build"
docker_cmd "${COMPOSE[@]}" up -d --build

wait_http() {
  local url="$1" label="$2" n=0
  echo -n "    waiting ${label} (${url})"
  while [[ "${n}" -lt 90 ]]; do
    if curl -sf -o /dev/null "${url}"; then
      echo " OK"
      return 0
    fi
    echo -n "."
    sleep 2
    n=$((n + 1))
  done
  echo " FAIL"
  return 1
}

echo "==> health"
HEALTH=0
wait_http "http://127.0.0.1:${API_PORT}/health" "backend /health" || HEALTH=1
wait_http "http://127.0.0.1:${FRONTEND_PORT}/" "frontend /" || HEALTH=1

echo "==> docker compose ps"
docker_cmd "${COMPOSE[@]}" ps

echo
echo "==> URLs"
echo "    frontend  http://127.0.0.1:${FRONTEND_PORT}/"
echo "    backend   http://127.0.0.1:${API_PORT}/health"
echo "    launcher  OWYX_API_BASE_URL=http://127.0.0.1:${API_PORT}"
if [[ "${DEV_MODE}" -eq 1 ]]; then
  echo "    adminer   http://127.0.0.1:${ADMINER_PORT}/  (debug)"
  echo "    turnstile NODE_ENV=development — TURNSTILE_SKIP=true is honored"
else
  echo "    bind      loopback only — put host nginx in front (deploy/nginx-owyx.site.conf)"
  echo "    panel     panel.owyx.site is NOT managed here (C³ CELERITY)"
fi

if [[ "${HEALTH}" -ne 0 ]]; then
  echo "error: health check failed — see docker compose logs" >&2
  exit 1
fi
echo "==> ready"
