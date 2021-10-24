FROM rust:1.56

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/daedalus_client"]