# syntax=docker/dockerfile:1

FROM docker.io/tailscale/tailscale:latest AS tailscale

FROM node:24-trixie-slim

LABEL org.opencontainers.image.source=https://github.com/modrinth/code
LABEL org.opencontainers.image.title=frontend
LABEL org.opencontainers.image.description="Modrinth website"
LABEL org.opencontainers.image.licenses=AGPL-3.0-only

RUN apt-get update \
	&& apt-get install -y --no-install-recommends dumb-init \
	&& rm -rf /var/lib/apt/lists/*

# Debug
COPY --from=tailscale /usr/local/bin/tailscale /usr/local/bin/tailscaled /usr/local/bin/

COPY --chmod=0755 <<'EOF' /usr/local/bin/entrypoint.sh
#!/bin/sh
set -eu

TAILSCALE_DIR=/tmp/tailscale
TAILSCALE_SOCKET="$TAILSCALE_DIR/tailscaled.sock"

start_tailscale() {
	echo "entrypoint: starting tailscale as euid=$(id -u) groups=$(id -G)" >&2
	mkdir -p "$TAILSCALE_DIR"

	# Userspace networking needs no TUN device or extra capabilities.
	# --state=mem: registers an ephemeral node; --statedir gives the SSH host keys
	# somewhere writable to live, which mem: on its own does not.
	tailscaled \
		--tun=userspace-networking \
		--state=mem: \
		--statedir="$TAILSCALE_DIR" \
		--socket="$TAILSCALE_SOCKET" &

	waited=0
	while [ ! -S "$TAILSCALE_SOCKET" ]; do
		if [ "$waited" -ge 50 ]; then
			echo "entrypoint: tailscaled socket never appeared" >&2
			return 1
		fi
		waited=$((waited + 1))
		sleep 0.1
	done

	if [ -n "${TAILSCALE_HOSTNAME:-}" ]; then
		node_hostname="$TAILSCALE_HOSTNAME"
	elif [ -n "${BUNNYNET_MC_PODID:-}" ] && [ -n "${BUNNYNET_MC_REGION:-}" ]; then
		node_hostname="frontend-$BUNNYNET_MC_PODID-$BUNNYNET_MC_REGION"
	else
		node_hostname="frontend-$(hostname)"
	fi

	tailscale --socket="$TAILSCALE_SOCKET" up \
		--authkey="$TAILSCALE_AUTH_KEY" \
		--hostname="$node_hostname" \
		--accept-dns=false \
		--timeout=30s \
		--ssh
}

if [ -n "${TAILSCALE_AUTH_KEY:-}" ]; then
	start_tailscale || echo "entrypoint: tailscale setup failed, serving without SSH" >&2
fi

# Only drop if we actually started as root; some runtimes pin their own uid.
if [ "$(id -u)" = "0" ]; then
	exec setpriv --reuid=node --regid=node --init-groups -- "$@"
fi

exec "$@"
EOF

ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=3000

WORKDIR /app

# Nitro bundles every runtime dependency into .output, so no node_modules is needed.
COPY --chown=node:node .output ./.output

# Stays root so tailscaled can setgroups/setuid for SSH sessions; the entrypoint
# drops the server itself to the node user.
EXPOSE 3000

ENTRYPOINT ["dumb-init", "--", "/usr/local/bin/entrypoint.sh"]
CMD ["node", "/app/.output/server/index.mjs"]
