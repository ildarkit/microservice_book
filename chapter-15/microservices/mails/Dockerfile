FROM rust:1.66 as builder

RUN USER=root cargo new mails-microservice
WORKDIR /mails-microservice
COPY Cargo.toml .
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
COPY ./templates ./templates
RUN rm ./target/debug/deps/mails_microservice*
RUN cargo build

FROM debian:buster-slim
RUN apt-get update && apt-get upgrade -y && rm -rf /var/lib/apt/lists/* 
COPY --from=builder /mails-microservice/target/debug/mails-microservice /app/
ENV RUST_LOG=debug
CMD ["/app/mails-microservice"]
EXPOSE 8000
