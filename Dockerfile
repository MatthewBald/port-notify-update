FROM rust:1.85 as builder
WORKDIR /usr/src/file-watcher
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/file-watcher /usr/local/bin/file-watcher
CMD ["file-watcher"]