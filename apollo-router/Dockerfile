FROM rust:1.64 as builder

ENV CARGO_TERM_COLOR always
RUN rustup component add rustfmt
RUN apt-get update && apt-get install -y npm

WORKDIR /usr/src/docker-build
# create empty project for caching dependencies
RUN USER=root cargo init
COPY common-utils ../common-utils
COPY Cargo.lock apollo-router/Cargo.toml ./
# cache dependencies
RUN cargo install --path . --locked

COPY apollo-router/ ./
RUN touch src/main.rs
RUN cargo install --path . --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
RUN update-ca-certificates
COPY --from=builder /usr/local/cargo/bin/apollo-router /bin/
CMD ["apollo-router"]
