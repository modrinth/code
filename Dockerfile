FROM rust:1.65.0 as build
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
# Add the wait script
ADD https://github.com/ufoscout/docker-compose-wait/releases/download/2.2.1/wait /wait
RUN chmod +x /wait
# Build our code
ARG SQLX_OFFLINE=true
RUN cargo build --release


FROM debian:bullseye-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

COPY --from=build /usr/src/labrinth/target/release/labrinth /labrinth/labrinth
COPY --from=build /usr/src/labrinth/migrations/* /labrinth/migrations/
COPY --from=build /wait /wait
WORKDIR /labrinth

CMD /wait && /labrinth/labrinth
