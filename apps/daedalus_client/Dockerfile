FROM rust:1.88.0 AS build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/daedalus
COPY . .
RUN cargo build --release --package daedalus_client


FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates openssl \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

COPY --from=build /usr/src/daedalus/target/release/daedalus_client /daedalus/daedalus_client
WORKDIR /daedalus_client

CMD /daedalus/daedalus_client
