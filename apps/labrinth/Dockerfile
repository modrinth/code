FROM rust:1.84.0 as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/labrinth
COPY . .
RUN cargo build --release


FROM debian:bookworm-slim

LABEL org.opencontainers.image.source=https://github.com/modrinth/code
LABEL org.opencontainers.image.description="Modrinth API"
LABEL org.opencontainers.image.licenses=AGPL-3.0

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates openssl \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

COPY --from=build /usr/src/labrinth/target/release/labrinth /labrinth/labrinth
COPY --from=build /usr/src/labrinth/migrations/* /labrinth/migrations/
COPY --from=build /usr/src/labrinth/assets /labrinth/assets
WORKDIR /labrinth

CMD /labrinth/labrinth