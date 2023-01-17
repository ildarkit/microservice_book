FROM rust:1.66 as builder

RUN USER=root cargo new mails-microservice
WORKDIR /mails-microservice
COPY Cargo.toml .
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
RUN rm ./target/debug/deps/mails_microservice*
RUN cargo build

FROM debian:bullseye-slim
WORKDIR /app
COPY ./templates ./templates
COPY --from=builder /mails-microservice/target/debug/mails-microservice ./
ENV RUST_LOG=debug
CMD ["./mails-microservice"]
EXPOSE 8000
