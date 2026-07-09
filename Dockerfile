FROM rust:1.89-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libsqlite3-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests

RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-taskflow-api /usr/local/bin/rust-taskflow-api

ENV HOST=0.0.0.0
ENV PORT=3000
ENV DATABASE_URL=sqlite://taskflow.db
ENV API_KEY=dev-secret-key

EXPOSE 3000

CMD ["rust-taskflow-api"]