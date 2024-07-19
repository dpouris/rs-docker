FROM rust:1.63-buster

RUN mkdir /app
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock

WORKDIR /app
ADD . .

RUN cargo build --release

ENTRYPOINT ["/app/target/release/rs-docker"]
