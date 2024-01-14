FROM rust:1.75.0 as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/labrinth
# Download and compile deps
COPY Cargo.toml .
COPY Cargo.lock .
COPY docker_utils/dummy.rs .
# Change temporarely the path of the code
RUN sed -i 's|src/main.rs|dummy.rs|' Cargo.toml
# Build only deps
RUN cargo build --release
# Now return the file back to normal
RUN sed -i 's|dummy.rs|src/main.rs|' Cargo.toml

# Copy everything
COPY . .
# Build our code
ARG SQLX_OFFLINE=true
RUN cargo build --release

# Final Stage
FROM ubuntu:latest

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

COPY --from=build /usr/src/labrinth/target/release/labrinth /labrinth/labrinth
COPY --from=build /usr/src/labrinth/migrations/* /labrinth/migrations/
COPY --from=build /usr/src/labrinth/assets /labrinth/assets
WORKDIR /labrinth

CMD /labrinth/labrinth