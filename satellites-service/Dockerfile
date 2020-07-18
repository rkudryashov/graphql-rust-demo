FROM rust:1.45.0

ENV CARGO_TERM_COLOR always
RUN apt-get update && apt-get install -y libpq-dev

# create empty project for caching dependencies
RUN USER=root cargo new --bin satellites-service
WORKDIR ./satellites-service
COPY ./Cargo.toml ./Cargo.toml
# cache dependencies
RUN cargo install --path .

COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libpq-dev
COPY --from=0 /usr/local/cargo/bin/satellites-service /usr/local/bin/satellites-service
CMD ["satellites-service"]
