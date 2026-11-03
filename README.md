# 文迅

桌面文章编辑与微信公众号发布工具，基于 Tauri 2 + React 18 + Rust 构建。

<p align="center">
  <img src="wenxun.jpg" alt="文迅" width="160" />
</p>

## 功能

- **文章编辑器** — Markdown 编辑，实时预览，智能排版
- **AI 生成** — 接入 DeepSeek API（自带 Key），话题生成、文章撰写、标题创作
- **爆文工坊** — 拆解爆款文章结构，辅助创作原创内容
- **热点雷达** — 抓取多平台热点话题，辅助选题
- **虚拟博主** — 内置多种写作人设，支持自定义风格
- **定时任务** — 单次/每天/每周/每月调度，AI 自动生成并发布文章到微信公众号
- **微信公众号** — 发布到草稿箱或群发，支持素材上传，发布记录追踪
- **本地存储** — 数据与日志全部落在本机，不上传任何服务器

## 技术栈

- 前端：React 18 + TypeScript + Ant Design + Vite + Zustand
- 后端：Rust + Tauri 2 + SQLite + Tokio
- AI：DeepSeek API（用户自备 API Key）
- 发布：微信公众平台 API（用户自备 AppID / AppSecret）

## 环境要求

- Node.js ≥ 18
- Rust ≥ 1.70（含 MSVC 工具链，Windows）
- Windows 10/11（打包目标平台）

## 开发

```bash
npm install
npm run tauri:dev
```

## 打包

```bash
npm run tauri:build
```

安装包输出在 `src-tauri/target/release/bundle/nsis/`，并自动复制到根目录 `release/`。

## 测试

```bash
npm test          # 前端单测（Jest）
npm run lint      # ESLint
```

## 项目结构

```
├── src/            # React 前端（页面 / 组件 / 服务）
├── src-tauri/      # Rust 后端（Tauri 命令 / 微信 API / 定时调度）
├── scripts/        # 构建后处理脚本
└── tests/          # 测试脚本与配置
```

## 免责声明

1. 本工具为辅助创作工具，AI 生成内容仅供参考，发布前请人工审核。
2. 发布到微信公众号的内容需遵守《微信公众平台运营规范》，违规后果由用户自行承担。
3. 请勿将本工具用于垃圾营销、批量灌水等滥用场景。

## 致谢

- [yotta-humanize](https://github.com/YottaMeta/yotta-humanize)（MIT）— 本地 AI 特征扫描的部分词表与改写规则移植自此项目

## License

[MIT](LICENSE)
