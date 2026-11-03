# Run all tests script
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Wenxun Desktop - Complete Test Suite" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"
$testResults = @()
$startTime = Get-Date

# 1. Frontend tests
Write-Host "[1/2] Running frontend tests..." -ForegroundColor Yellow
& "$PSScriptRoot\run-frontend.ps1"
if ($LASTEXITCODE -eq 0) {
    $testResults += @{Name="Frontend Tests"; Status="PASS"}
} else {
    $testResults += @{Name="Frontend Tests"; Status="FAIL"}
}

Write-Host ""
Write-Host "----------------------------------------" -ForegroundColor Gray
Write-Host ""

# 2. Backend tests
Write-Host "[2/2] Running backend tests..." -ForegroundColor Yellow
& "$PSScriptRoot\run-backend.ps1"
if ($LASTEXITCODE -eq 0) {
    $testResults += @{Name="Backend Tests"; Status="PASS"}
} else {
    $testResults += @{Name="Backend Tests"; Status="FAIL"}
}

# Output test results summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Test Results Summary" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$endTime = Get-Date
$duration = ($endTime - $startTime).TotalSeconds

foreach ($result in $testResults) {
    $statusColor = if ($result.Status -eq "PASS") { "Green" } else { "Red" }
    $statusIcon = if ($result.Status -eq "PASS") { "[PASS]" } else { "[FAIL]" }
    Write-Host "$statusIcon $($result.Name): $($result.Status)" -ForegroundColor $statusColor
}

Write-Host ""
Write-Host "Total time: $([math]::Round($duration, 2)) seconds" -ForegroundColor White

$failedTests = ($testResults | Where-Object { $_.Status -eq "FAIL" }).Count
if ($failedTests -eq 0) {
    Write-Host ""
    Write-Host "[SUCCESS] All tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host ""
    Write-Host "[ERROR] $failedTests test suite(s) failed" -ForegroundColor Red
    exit 1
}
