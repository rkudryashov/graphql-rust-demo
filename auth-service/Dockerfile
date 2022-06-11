FROM rust:1.61

ENV CARGO_TERM_COLOR always
RUN apt-get update && apt-get install -y libpq-dev clang

# create empty project for caching dependencies
RUN USER=root cargo new --bin /auth-service/docker-build
COPY /common-utils/ ./auth-service/common-utils/
WORKDIR /auth-service/docker-build
COPY /Cargo.lock ./
COPY /auth-service/Cargo.toml ./
# cache dependencies
RUN cargo install --path . --locked

COPY /auth-service/ ./
RUN cargo install --path . --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq-dev curl
COPY --from=0 /usr/local/cargo/bin/auth-service /usr/local/bin/auth-service
CMD ["auth-service"]
EXPOSE 8003
