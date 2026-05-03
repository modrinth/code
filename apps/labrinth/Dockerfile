# syntax=docker/dockerfile:1

FROM debian:trixie-slim

LABEL org.opencontainers.image.source=https://github.com/modrinth/code
LABEL org.opencontainers.image.title=labrinth
LABEL org.opencontainers.image.description="Modrinth API"
LABEL org.opencontainers.image.licenses=AGPL-3.0-only

RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates dumb-init curl \
	&& rm -rf /var/lib/apt/lists/*

COPY labrinth /labrinth/labrinth
COPY migrations /labrinth/migrations
COPY assets /labrinth/assets

WORKDIR /labrinth
ENTRYPOINT ["dumb-init", "--"]
CMD ["/labrinth/labrinth"]
