FROM rust:1.65

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/daedalus_client"]