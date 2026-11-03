# Frontend test runner
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Running Frontend Tests" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"

# Check dependencies
Write-Host "Checking dependencies..." -ForegroundColor Yellow
npm list jest > $null 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Installing test dependencies..." -ForegroundColor Yellow
    npm install --save-dev jest-junit
}

# Run tests
Write-Host ""
Write-Host "Running Jest tests..." -ForegroundColor Green
npm test

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "[SUCCESS] Frontend tests passed" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[ERROR] Frontend tests failed" -ForegroundColor Red
    exit 1
}
