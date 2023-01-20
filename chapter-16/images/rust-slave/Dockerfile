FROM jetbrains/teamcity-minimal-agent:latest
RUN apt update && apt install -y build-essential

ENV RUST_VERSION=1.66
RUN curl https://sh.rustup.rs -sSf \
| sh -s -- -y --no-modify-path --default-toolchain $RUST_VERSION
ENV PATH=/root/.cargo/bin:$PATH

RUN rustup --version; \
  cargo --version; \
  rustc --version;

RUN rustup component add rustfmt
RUN rustup component add clippy
