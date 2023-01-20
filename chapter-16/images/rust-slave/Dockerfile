FROM jetbrains/teamcity-minimal-agent:latest
USER root
RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential && \
    apt-get clean && rm -rf /var/lib/apt/lists/* && \
    rm -rf /var/lib/apt/lists/*

USER buildagent
ENV RUST_VERSION=1.66
RUN curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf \
| sh -s -- -y --no-modify-path --default-toolchain $RUST_VERSION
ENV PATH=/home/buildagent/.cargo/bin:$PATH
RUN rustup --version; \
  cargo --version; \
  rustc --version;

RUN rustup component add rustfmt
RUN rustup component add clippy
