# syntax=docker/dockerfile:1

FROM oven/bun:1.3.14 AS web-builder
WORKDIR /build/apps/web

COPY apps/web/package.json apps/web/bun.lock ./
RUN bun install --frozen-lockfile

COPY apps/web/ ./
RUN bun run build

FROM rust:1.94-bookworm AS api-builder
WORKDIR /build/apps/api

COPY apps/api/Cargo.toml apps/api/Cargo.lock ./
COPY apps/api/migrations ./migrations
COPY apps/api/queries ./queries
COPY apps/api/src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --uid 10001 kakeibo \
    && mkdir --parents /app/public /data \
    && chown --recursive kakeibo:kakeibo /app /data

COPY --from=api-builder /build/apps/api/target/release/kakeibo-app /app/kakeibo-app
COPY --from=web-builder /build/apps/web/dist /app/public

USER kakeibo
WORKDIR /data

ENV RUST_LOG=info
EXPOSE 8000
VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl --fail --silent http://127.0.0.1:8000/health > /dev/null || exit 1

ENTRYPOINT ["/app/kakeibo-app"]
