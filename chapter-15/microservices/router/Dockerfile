FROM rust:1.66

RUN USER=root cargo new router-microservice
WORKDIR /router-microservice
COPY Cargo.toml .
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
COPY ./static ./static
RUN rm ./target/debug/deps/router_microservice*
RUN cargo build

CMD ["./target/debug/router-microservice"]
EXPOSE 8000
