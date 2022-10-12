FROM rust:1.64-buster

WORKDIR /ublog
COPY . .

RUN cp ./ustc-cargo-mirror ~/.cargo/config
RUN cargo install --path .

WORKDIR /ublog-site
ENTRYPOINT ["ublog"]
