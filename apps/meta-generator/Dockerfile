# syntax=docker/dockerfile:1

FROM debian:trixie-slim

LABEL org.opencontainers.image.source=https://github.com/modrinth/code
LABEL org.opencontainers.image.title=modrinth-minecraft-meta
LABEL org.opencontainers.image.description="Modrinth Minecraft launcher metadata"
LABEL org.opencontainers.image.licenses=AGPL-3.0-only

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates dumb-init curl \
    && rm -rf /var/lib/apt/lists/*

COPY --chmod=0755 modrinth-minecraft-meta /modrinth-minecraft-meta/modrinth-minecraft-meta

WORKDIR /modrinth-minecraft-meta
ENTRYPOINT ["dumb-init", "--", "/modrinth-minecraft-meta/modrinth-minecraft-meta"]
