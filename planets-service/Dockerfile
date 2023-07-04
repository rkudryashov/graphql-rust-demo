FROM rust:1.70.0 as builder

ENV CARGO_TERM_COLOR always
RUN apt-get update && apt-get install -y libpq-dev cmake

WORKDIR /usr/src/docker-build
# create empty project for caching dependencies
RUN USER=root cargo init
COPY common-utils ../common-utils
COPY Cargo.lock planets-service/Cargo.toml ./
# cache dependencies
RUN cargo install --path . --locked

COPY planets-service/ ./
RUN touch src/main.rs
RUN cargo install --path . --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq-dev curl
COPY --from=builder /usr/local/cargo/bin/planets-service /bin/
CMD ["planets-service"]
