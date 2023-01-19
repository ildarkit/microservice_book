FROM rust:1.66 as builder

RUN USER=root cargo new router-microservice
WORKDIR /router-microservice
COPY Cargo.toml .
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
RUN rm ./target/debug/deps/router_microservice*
RUN cargo build

FROM debian:bullseye-slim
WORKDIR /app
COPY ./static ./static
COPY --from=builder /router-microservice/target/debug/router-microservice ./
ENV RUST_LOG=debug
CMD ["./router-microservice"]
EXPOSE 8000
