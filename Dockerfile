FROM rust:latest as builder

WORKDIR /usr/src/my-web-socket-chat
COPY src src
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

RUN cargo build --release

FROM gcr.io/distroless/cc-debian10

COPY --from=builder /usr/src/my-web-socket-chat/target/release/my-web-socket-chat /usr/local/bin/my-web-socket-chat

WORKDIR /usr/local/bin
CMD ["my-web-socket-chat"]