# syntax=docker/dockerfile:1

FROM debian:trixie-slim

LABEL org.opencontainers.image.source=https://github.com/modrinth/code
LABEL org.opencontainers.image.title=minecraft-meta
LABEL org.opencontainers.image.description="Modrinth Minecraft launcher metadata"
LABEL org.opencontainers.image.licenses=AGPL-3.0-only

RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates dumb-init \
	&& rm -rf /var/lib/apt/lists/* \
	&& groupadd --system app \
	&& useradd --system --gid app --create-home app

COPY --chmod=0755 meta-generator /minecraft-meta/meta-generator

USER app
WORKDIR /minecraft-meta
ENTRYPOINT ["dumb-init", "--", "/minecraft-meta/meta-generator"]
