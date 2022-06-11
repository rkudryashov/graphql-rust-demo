FROM rust:1.61

ENV CARGO_TERM_COLOR always
RUN apt-get update && apt-get install -y libpq-dev cmake

# create empty project for caching dependencies
RUN USER=root cargo new --bin /planets-service/docker-build
COPY /common-utils/ ./planets-service/common-utils/
WORKDIR /planets-service/docker-build
COPY /Cargo.lock ./
COPY /planets-service/Cargo.toml ./
# cache dependencies
RUN cargo install --path . --locked

COPY /planets-service/ ./
RUN cargo install --path . --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq-dev curl
COPY --from=0 /usr/local/cargo/bin/planets-service /usr/local/bin/planets-service
CMD ["planets-service"]
EXPOSE 8001
