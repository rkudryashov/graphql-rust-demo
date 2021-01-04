FROM rust:1.49.0

ENV CARGO_TERM_COLOR always
RUN apt-get update && apt-get install -y libpq-dev

# create empty project for caching dependencies
RUN USER=root cargo new --bin /satellites-service/docker-build
WORKDIR /satellites-service/docker-build
COPY /Cargo.lock ./
COPY /satellites-service/Cargo.toml ./
# cache dependencies
RUN cargo install --path . --locked

COPY /satellites-service/ ./
RUN cargo install --path . --locked

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libpq-dev curl
COPY --from=0 /usr/local/cargo/bin/satellites-service /usr/local/bin/satellites-service
CMD ["satellites-service"]
EXPOSE 8002
