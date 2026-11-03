# Backend test runner
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Running Backend Rust Tests" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"

# Switch to Rust project directory
Push-Location src-tauri

# Check Rust toolchain
Write-Host "Checking Rust toolchain..." -ForegroundColor Yellow
rustc --version > $null 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Rust toolchain not found, please install Rust first" -ForegroundColor Red
    Pop-Location
    exit 1
}

# Run tests
Write-Host ""
Write-Host "Running Cargo tests..." -ForegroundColor Green
cargo test --lib

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "[SUCCESS] Backend tests passed" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[ERROR] Backend tests failed" -ForegroundColor Red
    Pop-Location
    exit 1
}

Pop-Location
