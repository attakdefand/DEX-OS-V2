Param()
$ErrorActionPreference = 'Stop'
Write-Host "[dex-os] Building and starting services (Docker Compose)" -ForegroundColor Cyan
docker compose build
docker compose up -d
Write-Host "[dex-os] Waiting for API to be ready at http://localhost:3030" -ForegroundColor Cyan
Start-Sleep -Seconds 3
docker compose logs --no-log-prefix --tail=100 api | Out-Host
Write-Host "[dex-os] Done. Stop with: docker compose down" -ForegroundColor Green

