FROM rust:1.85 
WORKDIR /usr/src/file-watcher
COPY . .
RUN cargo install --path .
CMD ["file-watcher"]