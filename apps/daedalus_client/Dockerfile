FROM rust:1.88.0 AS build

WORKDIR /usr/src/daedalus
COPY . .
RUN cargo build --release --package daedalus_client


FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates openssl \
  && rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/src/daedalus/target/release/daedalus_client /daedalus/daedalus_client
WORKDIR /daedalus_client

CMD /daedalus/daedalus_client
