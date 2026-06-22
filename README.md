# Meet-God

开源的 AI 面试辅助桌面应用。实时捕获系统音频，语音转文字，AI 生成参考答案，悬浮窗展示。

> ⚠️ 本工具仅供技术学习和研究使用。用户在使用本工具时，应遵守所在地区的法律法规以及面试方的相关规定。开发者不对因使用本工具产生的任何后果承担责任。

## 📥 下载

前往 [Releases](../../releases) 页面下载最新版本：

- **Windows**: `.msi` 或 `.exe` 安装包
- **macOS / Linux**: 开发中

> 首次启动后需在设置中配置 LLM 的 API Key 才能使用 AI 答案生成功能。

## ✨ 核心特性

- 🎙️ **系统音频捕获** — 基于 WASAPI Loopback，无需虚拟声卡，直接捕获系统音频输出
- 🗣️ **语音识别** — 内置 Whisper 本地模型（零配置可用），同时支持云端 STT API
- 🤖 **AI 答案生成** — 支持 OpenAI 格式和 Anthropic 格式，用户自行配置 API URL 和 Key
- 👻 **极致隐蔽** — 窗口对屏幕共享、截图、录屏不可见，全局快捷键一键隐藏
- 🔒 **隐私优先** — 所有数据本地存储，不内置后端服务，API Key 由用户自行配置
- 📝 **面试记录** — 自动记录面试过程，支持回溯和导出

## 🏗️ 技术栈

| 层级 | 技术 |
|------|------|
| 框架 | [Tauri 2](https://v2.tauri.app/) |
| 后端 | Rust（音频捕获、STT、LLM 调用） |
| 前端 | React + TypeScript + Vite |
| 音频 | WASAPI Loopback (cpal) |
| STT | Whisper (whisper-rs) + 云端 API |
| LLM | OpenAI 格式 / Anthropic 格式 |
| 存储 | SQLite (rusqlite) |

## 🚀 从源码构建

### 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.77
- Windows 10 2004+ (Build 19041+)
- （可选）[LLVM/Clang](https://github.com/llvm/llvm-project/releases) — 启用本地 Whisper 模型
- （可选）[CMake](https://cmake.org/download/) — whisper-rs 编译依赖

### 安装依赖

```bash
npm install
```

### 开发模式（基础，不含本地 Whisper）

```bash
npm run tauri dev
```

### 开发模式（含本地 Whisper）

```bash
npm run tauri dev -- --features whisper-local
```

需要先安装 LLVM 和 CMake，并设置环境变量：
```powershell
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
```

### 构建

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/`。

## ⚙️ 配置说明

应用启动后，在设置页面中配置：

### 语音识别 (STT)

- **本地 Whisper**：首次启动自动下载模型（~150MB），零网络依赖
- **云端 API**：支持 OpenAI Whisper API 等，需填写 API Key 和 URL

### 大模型 (LLM)

支持两种 API 格式，用户自行填写 URL 和 Key：

| API 格式 | 适用模型 |
|---------|---------|
| **OpenAI 格式** | OpenAI、DeepSeek、通义千问、智谱、Ollama 等兼容 `/v1/chat/completions` 的服务 |
| **Anthropic 格式** | Claude 系列（`/v1/messages`） |

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+Shift+H` | 显示/隐藏答案窗 |
| `Ctrl+Shift+M` | 静音/取消静音 |
| `Ctrl+Shift+Esc` | 紧急退出 |

## 📁 项目结构

```
meet-god/
├── src/                  # React 前端
├── src-tauri/            # Rust 后端
│   ├── src/
│   │   ├── audio/        # 音频捕获模块
│   │   ├── stt/          # 语音识别模块
│   │   ├── llm/          # 大模型调用模块
│   │   ├── stealth/      # 隐蔽控制模块
│   │   ├── config/       # 配置管理
│   │   ├── recorder/     # 面试记录
│   │   └── pipeline/     # 数据管线
│   └── Cargo.toml
├── models/               # Whisper 模型文件（gitignore）
└── docs/                 # 文档
```

## 📦 自动发布

推送 `v*` 格式的 tag 会自动触发 GitHub Actions 构建安装包：

```bash
git tag v0.1.0
git push origin v0.1.0
```

构建完成后会自动在 [Releases](../../releases) 页面发布安装包。

## 📄 开源协议

[MIT License](LICENSE)
