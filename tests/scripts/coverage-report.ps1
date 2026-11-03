# 生成测试覆盖率报告
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "生成测试覆盖率报告" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"

# 前端覆盖率
Write-Host "[1/2] 生成前端覆盖率报告..." -ForegroundColor Yellow
npm run test:coverage

if (Test-Path "coverage/frontend/lcov.info") {
    Write-Host "[SUCCESS] 前端覆盖率报告已生成: coverage/frontend/" -ForegroundColor Green
} else {
    Write-Host "[ERROR] 前端覆盖率报告生成失败" -ForegroundColor Red
}

Write-Host ""
Write-Host "----------------------------------------" -ForegroundColor Gray
Write-Host ""

# 后端覆盖率（需要安装 cargo-tarpaulin）
Write-Host "[2/2] 检查后端覆盖率工具..." -ForegroundColor Yellow

Push-Location src-tauri

$cargoTarpaulin = cargo install --list | Select-String "cargo-tarpaulin"
if ($cargoTarpaulin) {
    Write-Host "运行 Cargo Tarpaulin..." -ForegroundColor Green
    cargo tarpaulin --out Html --output-dir ../coverage/backend
    
    if (Test-Path "../coverage/backend/tarpaulin-report.html") {
        Write-Host "[SUCCESS] 后端覆盖率报告已生成: coverage/backend/" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] 后端覆盖率报告生成失败" -ForegroundColor Red
    }
} else {
    Write-Host "[WARNING] cargo-tarpaulin 未安装" -ForegroundColor Yellow
    Write-Host "  安装命令: cargo install cargo-tarpaulin" -ForegroundColor Gray
}

Pop-Location

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "覆盖率报告位置:" -ForegroundColor Cyan
Write-Host "  前端: coverage/frontend/index.html" -ForegroundColor White
Write-Host "  后端: coverage/backend/tarpaulin-report.html" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
