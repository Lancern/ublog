FROM rust:1.64-buster

WORKDIR /ublog
COPY . .

RUN mkdir -p $HOME/.cargo
RUN cp ./ustc-cargo-mirror $HOME/.cargo/config
RUN cargo install --path .

WORKDIR /ublog-site
ENTRYPOINT ["ublog"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://127.0.0.1:8000/api/posts || exit 1
