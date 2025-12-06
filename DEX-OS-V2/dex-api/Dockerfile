FROM rust:1-bullseye AS builder
WORKDIR /app

# Copy full workspace for a straightforward build; add a .dockerignore to trim if needed.
COPY . .
RUN cargo build --target x86_64-unknown-linux-gnu --release -p dex-api

FROM debian:bullseye-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
ENV RUST_LOG=info
ENV DATABASE_URL=postgres://user:password@localhost:5432/dex_os
ENV JWT_SECRET=change-me
ENV SERVER_PORT=3030
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/dex-api /usr/local/bin/dex-api
COPY --from=builder /app/.reference /app/.reference

EXPOSE 3030
CMD ["/usr/local/bin/dex-api"]
