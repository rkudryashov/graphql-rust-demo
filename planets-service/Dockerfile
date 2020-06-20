FROM rust:1.44.1

ENV CARGO_TERM_COLOR always
RUN apt-get update && apt-get install -y libpq-dev

# create empty project for caching dependencies
RUN USER=root cargo new --bin planets-service
WORKDIR ./planets-service
COPY ./Cargo.toml ./Cargo.toml
# cache dependencies
RUN cargo install --path .

COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libpq-dev
COPY --from=0 /usr/local/cargo/bin/planets-service /usr/local/bin/planets-service
CMD ["planets-service"]
