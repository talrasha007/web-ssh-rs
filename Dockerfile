# syntax=docker/dockerfile:1
FROM rust:1-slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential cmake clang pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/web-ssh-rs /usr/local/bin/web-ssh-rs
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/web-ssh-rs"]
