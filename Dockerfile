FROM rust:1.75.0 as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/labrinth
COPY . .
RUN cargo build --release


FROM debian:bookworm-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

COPY --from=build /usr/src/labrinth/target/release/labrinth /labrinth/labrinth
WORKDIR /labrinth

CMD /labrinth/labrinth