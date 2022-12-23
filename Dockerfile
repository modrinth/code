FROM rust:1.65-slim

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/daedalus_client"]