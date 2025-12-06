Param()
$ErrorActionPreference = 'Stop'
Write-Host "[dex-os] Setup and build (Windows)" -ForegroundColor Cyan

if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
  Write-Host "Installing rustup..." -ForegroundColor Yellow
  Invoke-WebRequest https://win.rustup.rs/x86_64 -OutFile "$env:TEMP\rustup-init.exe"
  Start-Process -FilePath "$env:TEMP\rustup-init.exe" -ArgumentList "-y" -Wait
  $env:PATH = "$env:USERPROFILE\.cargo;" + $env:PATH
}

rustup update stable
rustup default stable

Push-Location (Join-Path $PSScriptRoot "..\DEX-OS-V2")
try {
  Write-Host "[dex-os] Cleaning and building workspace..." -ForegroundColor Cyan
  cargo clean
  cargo build --workspace

  Write-Host "[dex-os] Building core only (optional)..." -ForegroundColor Cyan
  cargo build -p dex-core
  Write-Host "[dex-os] Testing core only (optional)..." -ForegroundColor Cyan
  cargo test -p dex-core
}
finally {
  Pop-Location
}

Write-Host "[dex-os] Done." -ForegroundColor Green

