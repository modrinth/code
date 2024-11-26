#FROM rust:1.82.0 as build
FROM rust1.82.0_build:latest  as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/labrinth
COPY . .
ARG SQLX_OFFLINE=true
RUN export CARGO_NET_TIMEOUT=10  # 将超时设置为60秒
RUN cargo build --release --features jemalloc



FROM debian:bookworm-slim

LABEL org.opencontainers.image.source=https://github.com/bbsmc/app
LABEL org.opencontainers.image.description="BBSMC API"
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