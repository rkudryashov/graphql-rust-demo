FROM rust:1.62

ENV CARGO_TERM_COLOR always
RUN rustup component add rustfmt
RUN apt-get update && apt-get install -y npm

# create empty project for caching dependencies
RUN USER=root cargo new --bin /apollo-router/docker-build
COPY /common-utils/ ./apollo-router/common-utils/
WORKDIR /apollo-router/docker-build
COPY /Cargo.lock ./
COPY /apollo-router/Cargo.toml ./
# cache dependencies
RUN cargo install --path . --locked

COPY /apollo-router/ ./
RUN cargo install --path . --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
RUN update-ca-certificates
COPY --from=0 /usr/local/cargo/bin/apollo-router /usr/local/bin/apollo-router
CMD ["apollo-router"]
