FROM rust:1.64-buster

WORKDIR /ublog
COPY . .

RUN cargo install --path .

WORKDIR /ublog-site
ENTRYPOINT ["ublog"]
