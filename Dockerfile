FROM rust:1.85 
WORKDIR /usr/src/file-watcher
COPY . .
RUN cargo install --path .

# FROM debian:bullseye-slim
# RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
# COPY --from=builder /usr/local/cargo/bin/file-watcher /usr/local/bin/file-watcher
CMD file-watcher ${FILEPATH}