FROM rustlang/rust:nightly-slim AS builder
ENV CARGO_UNSTABLE_EDITION2024=1
WORKDIR /app

# Leverage layer caching for dependencies
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# Copy real sources
COPY src ./src
COPY migrations ./migrations
COPY Cargo.lock ./

# Build release binary
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

ENV APP_PORT=3000
COPY --from=builder /app/target/release/backend /app/backend

EXPOSE 3000
CMD ["/app/backend"]

