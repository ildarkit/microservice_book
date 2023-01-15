FROM rust:1.66 as builder

RUN USER=root cargo new dbsync-worker
WORKDIR /dbsync-worker
COPY Cargo.toml .
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
COPY ./migrations ./migrations
COPY diesel.toml .
RUN rm ./target/debug/deps/dbsync_worker*
RUN cargo build

FROM debian:buster-slim
RUN apt-get update && apt-get upgrade -y && rm -rf /var/lib/apt/lists/* 
COPY --from=builder /dbsync-worker/target/debug/dbsync-worker /app/
ENV RUST_LOG=debug
EXPOSE 8000
ENTRYPOINT ["/app/dbsync-worker"]
