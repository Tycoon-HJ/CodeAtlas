<div align="center">

# 🗺️ CodeAtlas Studio

**通用 AI 编程工作台**

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/your-username/codeatlas-studio)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Vue](https://img.shields.io/badge/Vue-3.5-brightgreen.svg)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-6.0-blue.svg)](https://www.typescriptlang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-orange.svg)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)

一个现代化的 AI 编程工作台，集成多种 AI 助手，为开发者提供智能编码体验。

[功能特性](#-功能特性) •
[快速开始](#-快速开始) •
[技术栈](#-技术栈) •
[项目结构](#-项目结构) •
[开发指南](#-开发指南) •
[贡献指南](#-贡献指南)

</div>

---

## ✨ 功能特性

### 🤖 多 AI 提供商支持
- **Claude Code** - Anthropic 的 AI 编程助手
- **Codex** - OpenAI 的代码生成模型
- **Gemini** - Google 的多模态 AI
- **通义千问** - 阿里云的大语言模型
- **OpenCode** - 开源代码模型

### 💬 智能对话
- 流式响应输出
- 思考过程可视化
- 工具调用追踪
- 命令历史 (↑/↓ 浏览)
- CLI 风格界面

### 📁 项目管理
- 快速打开目录
- 项目搜索与收藏
- Git 分支显示
- 最近项目记录

### 🛠️ 开发工具
- **Monaco Editor** - 强大的代码编辑器
- **终端集成** - xterm.js 终端模拟
- **文件树** - 可视化项目结构
- **代码高亮** - 多语言语法高亮

### 🔌 扩展能力
- **MCP 支持** - Model Context Protocol
- **技能系统** - 可扩展的 AI 技能
- **工作流** - 自动化开发流程

### 🎨 用户体验
- 暗色/亮色主题
- 响应式布局
- 快捷键支持
- 中文本地化

---

## 🚀 快速开始

### 环境要求

- **Node.js** >= 18.0.0
- **Rust** >= 1.70.0
- **npm** >= 9.0.0 或 **yarn** >= 1.22.0

### 安装步骤

1. **克隆仓库**
   ```bash
   git clone https://github.com/your-username/codeatlas-studio.git
   cd codeatlas-studio
   ```

2. **安装依赖**
   ```bash
   npm install
   ```

3. **启动开发服务器**
   ```bash
   npm run tauri dev
   ```

4. **构建生产版本**
   ```bash
   npm run tauri build
   ```

---

## 🛠️ 技术栈

### 前端

| 技术 | 版本 | 用途 |
|------|------|------|
| [Vue 3](https://vuejs.org/) | 3.5 | 渐进式 JavaScript 框架 |
| [TypeScript](https://www.typescriptlang.org/) | 6.0 | 类型安全的 JavaScript |
| [Vite](https://vitejs.dev/) | 8.0 | 下一代前端构建工具 |
| [Naive UI](https://www.naiveui.com/) | 2.44 | Vue 3 组件库 |
| [Pinia](https://pinia.vuejs.org/) | 3.0 | Vue 状态管理 |
| [Vue Router](https://router.vuejs.org/) | 5.1 | Vue 路由管理 |
| [Monaco Editor](https://microsoft.github.io/monaco-editor/) | 0.55 | 代码编辑器 |
| [xterm.js](https://xtermjs.org/) | 6.0 | 终端模拟器 |
| [D3.js](https://d3js.org/) | 7.9 | 数据可视化 |

### 后端

| 技术 | 版本 | 用途 |
|------|------|------|
| [Tauri](https://tauri.app/) | 2.0 | 桌面应用框架 |
| [Rust](https://www.rust-lang.org/) | 2021 | 系统编程语言 |
| [SQLite](https://www.sqlite.org/) | - | 嵌入式数据库 |

### 开发工具

| 工具 | 用途 |
|------|------|
| [ESLint](https://eslint.org/) | 代码检查 |
| [Prettier](https://prettier.io/) | 代码格式化 |
| [SASS](https://sass-lang.com/) | CSS 预处理器 |

---

## 📁 项目结构

```
codeatlas-studio/
├── src/                    # 前端源码
│   ├── api/               # API 接口
│   ├── assets/            # 静态资源
│   ├── components/        # 组件
│   │   ├── common/       # 通用组件
│   │   └── layout/       # 布局组件
│   ├── router/            # 路由配置
│   ├── stores/            # Pinia 状态管理
│   │   ├── agent.ts      # AI 代理状态
│   │   ├── project.ts    # 项目状态
│   │   ├── session.ts    # 会话状态
│   │   └── settings.ts   # 设置状态
│   ├── styles/            # 全局样式
│   ├── types/             # TypeScript 类型定义
│   └── views/             # 页面视图
│       ├── agents/       # AI 代理页面
│       ├── capabilities/ # 能力中心
│       ├── sessions/     # 会话页面
│       ├── settings/     # 设置页面
│       └── workspace/    # 工作区页面
├── src-tauri/             # Tauri 后端源码
│   ├── src/              # Rust 源码
│   ├── Cargo.toml        # Rust 依赖配置
│   └── tauri.conf.json   # Tauri 配置
├── public/                # 公共资源
├── dist/                  # 构建输出
├── package.json           # Node.js 依赖
├── tsconfig.json          # TypeScript 配置
└── vite.config.ts         # Vite 配置
```

---

## 💻 开发指南

### 可用脚本

```bash
# 启动开发服务器
npm run dev

# 构建前端
npm run build

# 预览构建结果
npm run preview

# 启动 Tauri 开发模式
npm run tauri dev

# 构建 Tauri 应用
npm run tauri build
```

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl + Enter` / `Cmd + Enter` | 发送消息 |
| `↑` | 浏览上一条命令 |
| `↓` | 浏览下一条命令 |

### 配置 AI 提供商

在设置页面中配置您的 AI 提供商 API 密钥：

1. 打开设置页面
2. 选择 AI 提供商
3. 输入 API 密钥
4. 保存配置

---

## 📦 构建与发布

### 开发构建

```bash
npm run tauri dev
```

### 生产构建

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

### 支持平台

- ✅ Windows (x64)
- ✅ macOS (x64, arm64)
- ✅ Linux (x64)

---

## 🤝 贡献指南

我们欢迎所有形式的贡献！

### 如何贡献

1. **Fork** 本仓库
2. 创建您的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交您的更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开一个 **Pull Request**

### 开发规范

- 使用 TypeScript 编写前端代码
- 遵循 ESLint 和 Prettier 配置
- 编写清晰的提交信息
- 更新相关文档

### 问题反馈

如果您发现了 bug 或有功能建议，请 [创建 Issue](https://github.com/Tycoon-HJ/CodeAtlas/issues)。

---

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

---

## 🙏 致谢

感谢以下开源项目：

- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Tauri](https://tauri.app/) - 桌面应用框架
- [Naive UI](https://www.naiveui.com/) - Vue 3 组件库
- [Monaco Editor](https://microsoft.github.io/monaco-editor/) - 代码编辑器
- [xterm.js](https://xtermjs.org/) - 终端模拟器

---

<div align="center">

**[⬆ 回到顶部](#-codeatlas-studio)**

Made with ❤️ by [Tycoon](https://github.com/Tycoon-HJ)

</div>
