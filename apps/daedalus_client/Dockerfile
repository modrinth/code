# syntax=docker/dockerfile:1

FROM rust:1.89.0 AS build

WORKDIR /usr/src/daedalus
COPY . .
RUN --mount=type=cache,target=/usr/src/daedalus/target \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  --mount=type=cache,target=/usr/local/cargo/registry \
  cargo build --release --package daedalus_client

FROM build AS artifacts

RUN --mount=type=cache,target=/usr/src/daedalus/target \
  mkdir /daedalus \
  && cp /usr/src/daedalus/target/release/daedalus_client /daedalus/daedalus_client

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates openssl \
  && rm -rf /var/lib/apt/lists/*

COPY --from=artifacts /daedalus /daedalus

WORKDIR /daedalus_client
CMD ["/daedalus/daedalus_client"]
