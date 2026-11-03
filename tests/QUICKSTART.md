# 测试系统快速启动指南

## 🚀 5分钟快速开始

### 1. 运行所有测试

```powershell
npm run test:all
```

这将依次运行：
- ✅ 前端测试（Jest）
- ✅ 后端测试（Cargo）

### 2. 查看测试结果

测试完成后会显示：
```
========================================
测试结果汇总
========================================
✓ 前端测试: PASS
✓ 后端测试: PASS

总耗时: XX.XX 秒

✓ 所有测试通过！
```

## 📋 常用命令

### 前端测试

```powershell
# 运行所有前端测试
npm run test:frontend

# 监听模式（开发时使用）
npm run test:watch

# 生成覆盖率报告
npm run test:coverage
```

### 后端测试

```powershell
# 运行所有后端测试
npm run test:backend

# 或直接使用 cargo
cd src-tauri
cargo test
```

### 覆盖率报告

```powershell
# 生成前后端覆盖率报告
npm run test:coverage-report

# 查看报告
# 前端: coverage/frontend/index.html
# 后端: coverage/backend/tarpaulin-report.html
```

## 🧪 添加新测试

### 前端测试

在 `tests/frontend/unit/` 创建测试文件：

```typescript
// tests/frontend/unit/myFeature.test.ts
describe('My Feature', () => {
  it('should work correctly', () => {
    expect(true).toBe(true);
  });
});
```

或在组件旁边创建：

```typescript
// src/components/MyComponent.test.tsx
import { render, screen } from '@testing-library/react';
import MyComponent from './MyComponent';

describe('MyComponent', () => {
  it('should render', () => {
    render(<MyComponent />);
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });
});
```

### 后端测试

在源文件中添加测试模块：

```rust
// src/database.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        assert_eq!(2 + 2, 4);
    }
}
```

或在单独文件中：

```rust
// src/database/my_test.rs

#[cfg(test)]
mod tests {
    use crate::database::Database;

    #[test]
    fn test_database() {
        // 测试代码
    }
}
```

## 📊 测试文件位置

```
项目根目录/
├── src/                          # 前端源代码
│   ├── utils/
│   │   └── validation.test.ts   # 验证测试
│   └── components/
│       └── *.test.tsx           # 组件测试
│
├── tests/                        # 集中测试目录
│   ├── frontend/
│   │   ├── unit/                # 单元测试
│   │   ├── integration/         # 集成测试
│   │   └── e2e/                 # E2E测试
│   └── backend/
│       ├── unit/                # Rust单元测试
│       └── integration/         # Rust集成测试
│
└── src-tauri/                    # 后端源代码
    └── src/
        ├── database/
        │   └── database_test.rs # 数据库测试
        └── *.rs                 # 其他模块的测试
```

## 🔍 调试测试

### 前端

```powershell
# 运行单个测试文件
npm test -- src/utils/validation.test.ts

# 运行匹配的测试
npm test -- --testNamePattern="validateTitle"

# 详细输出
npm test -- --verbose
```

### 后端

```powershell
cd src-tauri

# 运行特定测试
cargo test test_database_creation

# 显示输出
cargo test -- --nocapture

# 运行特定模块
cargo test database
```

## ⚙️ 配置说明

### Jest 配置 (jest.config.js)

```javascript
{
  testMatch: [
    'src/**/*.test.{ts,tsx}',           // src中的测试
    'tests/frontend/**/*.test.{ts,tsx}', // tests目录的测试
  ],
  coverageDirectory: 'coverage/frontend',
  reporters: ['default', 'jest-junit']
}
```

### Cargo 配置 (Cargo.toml)

```toml
[lib]
name = "wenxun"
path = "src/lib.rs"

[dev-dependencies]
tokio-test = "0.4"  # 异步测试
mockall = "0.11"    # Mock对象
```

## 🎯 测试最佳实践

### 1. 命名规范

```typescript
// ✅ 好的命名
it('should return error when title is empty', () => { ... });
it('should save article successfully', () => { ... });

// ❌ 不好的命名
it('test1', () => { ... });
it('works', () => { ... });
```

### 2. AAA 模式

```typescript
it('should calculate total', () => {
  // Arrange - 准备
  const items = [{ price: 10 }, { price: 20 }];
  
  // Act - 执行
  const total = calculateTotal(items);
  
  // Assert - 断言
  expect(total).toBe(30);
});
```

### 3. 测试隔离

```typescript
// ✅ 每个测试独立
beforeEach(() => {
  cleanup(); // 清理之前的测试
});

// ❌ 避免测试依赖
it('test 1', () => { ... });
it('test 2 depends on test 1', () => { ... }); // 不要这样做
```

## 🐛 常见问题

### Q: 测试找不到模块？
A: 检查 `jest.config.js` 中的 `moduleNameMapper` 配置。

### Q: Rust 测试编译失败？
A: 确保 `lib.rs` 正确导出模块。

### Q: 如何跳过测试？
```typescript
it.skip('skip this test', () => { ... });
```

```rust
#[ignore]
#[test]
fn skip_this_test() { ... }
```

## 📚 更多资源

- [测试说明](./README.md)
- [Jest 文档](https://jestjs.io/)
- [Rust 测试文档](https://doc.rust-lang.org/book/ch11-00-testing.html)

## 💡 提示

1. **经常运行测试** - 每次修改代码后运行相关测试
2. **编写测试优先** - TDD（测试驱动开发）提高代码质量
3. **保持测试简洁** - 测试应该易于理解和维护
4. **覆盖边界情况** - 不仅测试正常流程，还要测试异常情况
5. **使用 Mock** - 隔离外部依赖，提高测试速度

---

**需要帮助？** 查看 [`tests/README.md`](./README.md) 获取详细说明。
