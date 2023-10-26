FROM rust:1.73.0 AS chef
WORKDIR soler
ARG GIT_REVISION
ENV GIT_REVISION=$GIT_REVISION
RUN apt-get update && apt-get install -y cmake clang

FROM chef AS builder 

COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN cargo build --release \
    --bin api

# Production Image
FROM debian:bullseye-slim AS runtime
WORKDIR soler
COPY --from=builder /soler/target/release/api /usr/local/bin
# api needs postgres libpq5 and ca-certificates
RUN apt update && apt install -y libpq5 ca-certificates

ARG BUILD_DATE
ARG GIT_REVISION
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION
