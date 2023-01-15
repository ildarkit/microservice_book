FROM rust:1.66 as builder

RUN USER=root cargo new content-microservice
WORKDIR /content-microservice
COPY Cargo.toml .
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
RUN rm ./target/debug/deps/content_microservice*
RUN cargo build

FROM debian:buster-slim
RUN apt-get update && apt-get upgrade -y && rm -rf /var/lib/apt/lists/* 
COPY --from=builder /content-microservice/target/debug/content-microservice /app/
ENV RUST_LOG=debug
CMD ["/app/content-microservice"]
EXPOSE 8000
