# Multi-stage build for DEX-OS-V2

FROM rust:1-bullseye AS builder
WORKDIR /app
COPY . .

# Build the Rust workspace (you can scope to specific package if needed)
WORKDIR /app/DEX-OS-V2
RUN cargo build --release -p dex-core --bin riskctl --bin compliance_report
RUN cargo build --release -p dex-api

FROM debian:bullseye-slim AS runtime
WORKDIR /app
COPY --from=builder /app/DEX-OS-V2/target/release/riskctl /usr/local/bin/riskctl
COPY --from=builder /app/DEX-OS-V2/target/release/compliance_report /usr/local/bin/compliance_report
COPY --from=builder /app/DEX-OS-V2/target/release/dex-api /usr/local/bin/dex-api

ENV RUST_LOG=info
EXPOSE 3030

# Utilities for healthchecks
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Default command prints help for available tools
CMD ["bash", "-lc", "echo 'Available: riskctl, compliance_report, dex-api'; which dex-api && echo 'Run: dex-api (requires DATABASE_URL)' && exec sleep infinity"]
