FROM rust:1.88.0 AS build

WORKDIR /usr/src/labrinth
COPY . .
RUN SQLX_OFFLINE=true cargo build --release --package labrinth

FROM debian:bookworm-slim

LABEL org.opencontainers.image.source=https://github.com/modrinth/code
LABEL org.opencontainers.image.description="Modrinth API"
LABEL org.opencontainers.image.licenses=AGPL-3.0

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates openssl dumb-init curl \
  && rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/src/labrinth/target/release/labrinth /labrinth/labrinth
COPY --from=build /usr/src/labrinth/apps/labrinth/migrations/* /labrinth/migrations/
COPY --from=build /usr/src/labrinth/apps/labrinth/assets /labrinth/assets
WORKDIR /labrinth

ENTRYPOINT ["dumb-init", "--"]
CMD ["/labrinth/labrinth"]
