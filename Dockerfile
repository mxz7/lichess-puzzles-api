# syntax=docker/dockerfile:1

FROM rust:1.100.0-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs sqlx.toml ./
COPY .cargo/config.toml ./.cargo/config.toml
COPY .sqlx/ ./.sqlx/
COPY src/ ./src/
RUN cargo build --release --locked

FROM debian:bookworm-slim
WORKDIR /app
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends ca-certificates
COPY --from=build /app/target/release/lichess-puzzles /usr/local/bin/lichess-puzzles

EXPOSE 3000
CMD ["lichess-puzzles"]
