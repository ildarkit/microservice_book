FROM rust:1.66 as builder

RUN USER=root cargo new router-microservice
WORKDIR /router-microservice
COPY Cargo.toml .
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
COPY ./static ./static
RUN rm ./target/debug/deps/router_microservice*
RUN cargo build

FROM debian:bullseye-slim
COPY --from=builder /router-microservice/target/debug/router-microservice /app/
ENV RUST_LOG=debug
CMD ["/app/router-microservice"]
EXPOSE 8000
