#!/usr/bin/env bash
set -euo pipefail

echo "[dex-os] Building and starting services (Docker Compose)"
docker compose build
docker compose up -d
echo "[dex-os] Waiting for API to be ready at http://localhost:3030"
sleep 3
docker compose logs --no-log-prefix --tail=100 api || true
echo "[dex-os] Done. Stop with: docker compose down"

