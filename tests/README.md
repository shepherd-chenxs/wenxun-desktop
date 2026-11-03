# 测试配置和工具

## 目录结构

```
tests/
├── frontend/          # 前端测试
│   ├── unit/         # 单元测试
│   ├── integration/  # 集成测试
│   └── e2e/          # 端到端测试
├── backend/          # 后端 Rust 测试
│   ├── unit/         # 单元测试
│   └── integration/  # 集成测试
└── scripts/          # 测试脚本
    ├── run-all.ps1   # 运行所有测试
    ├── run-frontend.ps1
    ├── run-backend.ps1
    └── coverage-report.ps1
```

## 运行测试

### 前端测试
```powershell
# 运行所有前端测试
npm test

# 监听模式
npm run test:watch

# 生成覆盖率报告
npm run test:coverage
```

### 后端测试
```powershell
# 运行所有 Rust 测试
cargo test

# 运行特定模块测试
cargo test --package wenxun --lib database

# 生成覆盖率报告（需要安装 cargo-tarpaulin）
cargo tarpaulin --out Html
```

### 运行所有测试
```powershell
# Windows PowerShell
.\tests\scripts\run-all.ps1
```
