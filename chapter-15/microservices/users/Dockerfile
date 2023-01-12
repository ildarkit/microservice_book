FROM rust:1.66

RUN USER=root cargo new users-microservice
WORKDIR /users-microservice
COPY Cargo.toml diesel.toml config.toml ./
COPY src/lib.rs src/models.rs src/schema.rs src/
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
RUN rm ./target/debug/deps/users_microservice*
RUN cargo build

CMD ["./target/debug/users-microservice"]
EXPOSE 8000
