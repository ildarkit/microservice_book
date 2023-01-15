FROM rust:1.66 as builder

RUN USER=root cargo new users-microservice
WORKDIR /users-microservice
COPY Cargo.toml diesel.toml config.toml ./
COPY src/lib.rs src/models.rs src/schema.rs src/
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
RUN rm ./target/debug/deps/users_microservice*
RUN cargo build

FROM debian:buster-slim
RUN apt-get update && apt-get upgrade -y && rm -rf /var/lib/apt/lists/* 
COPY --from=builder /users-microservice/target/debug/users-microservice /app/
ENV RUST_LOG=debug
CMD ["/app/users-microservice"]
EXPOSE 8000
