FROM rust:1.66

RUN USER=root cargo new content-microservice
WORKDIR /content-microservice
COPY Cargo.toml .
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
RUN rm ./target/debug/deps/content_microservice*
RUN cargo build

CMD ["./target/debug/content-microservice"]
EXPOSE 8000
