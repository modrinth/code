# syntax=docker/dockerfile:1

FROM rust:1.90.0 AS build

WORKDIR /usr/src/labrinth
COPY . .
ARG GIT_HASH
RUN --mount=type=cache,target=/usr/src/labrinth/target \
	--mount=type=cache,target=/usr/local/cargo,from=rust:1.89.0,source=/usr/local/cargo \
	SQLX_OFFLINE=true cargo build --profile release-labrinth --package labrinth

FROM build AS artifacts

RUN --mount=type=cache,target=/usr/src/labrinth/target \
	mkdir /labrinth \
	&& cp /usr/src/labrinth/target/release-labrinth/labrinth /labrinth/labrinth \
	&& cp -r /usr/src/labrinth/apps/labrinth/migrations /labrinth \
	&& cp -r /usr/src/labrinth/apps/labrinth/assets /labrinth

FROM debian:trixie-slim

LABEL org.opencontainers.image.source=https://github.com/modrinth/code
LABEL org.opencontainers.image.title=labrinth
LABEL org.opencontainers.image.description="Modrinth API"
LABEL org.opencontainers.image.licenses=AGPL-3.0-only

RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates dumb-init curl \
	&& rm -rf /var/lib/apt/lists/*

COPY --from=artifacts /labrinth /labrinth

WORKDIR /labrinth
ENTRYPOINT ["dumb-init", "--"]
CMD ["/labrinth/labrinth"]
