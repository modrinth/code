FROM rust:1.88.0 AS build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/labrinth
COPY . .
COPY apps/labrinth/.sqlx/ .sqlx/
RUN cargo build --release --package labrinth


FROM debian:bookworm-slim

LABEL org.opencontainers.image.source=https://github.com/modrinth/code
LABEL org.opencontainers.image.description="Modrinth API"
LABEL org.opencontainers.image.licenses=AGPL-3.0

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates openssl dumb-init \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

COPY --from=build /usr/src/labrinth/target/release/labrinth /labrinth/labrinth
COPY --from=build /usr/src/labrinth/apps/labrinth/migrations/* /labrinth/migrations/
COPY --from=build /usr/src/labrinth/apps/labrinth/assets /labrinth/assets
WORKDIR /labrinth

ENTRYPOINT ["dumb-init", "--"]
CMD ["/labrinth/labrinth"]
