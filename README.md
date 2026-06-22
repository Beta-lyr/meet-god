# Meet-God

开源的 AI 面试辅助桌面应用。实时捕获系统音频，语音转文字，AI 生成参考答案，悬浮窗展示。

> ⚠️ 本工具仅供技术学习和研究使用。用户在使用本工具时，应遵守所在地区的法律法规以及面试方的相关规定。开发者不对因使用本工具产生的任何后果承担责任。

## ✨ 核心特性

- 🎙️ **系统音频捕获** — 基于 WASAPI Loopback，无需虚拟声卡，直接捕获系统音频输出
- 🗣️ **语音识别** — 内置 Whisper 本地模型（零配置可用），同时支持云端 STT API
- 🤖 **AI 答案生成** — 支持 OpenAI / Claude / 通义千问 / DeepSeek / Ollama 等任意兼容 OpenAI 格式的大模型
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
| LLM | OpenAI 兼容格式，多 Provider |
| 存储 | SQLite (rusqlite) |

## 🚀 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.77
- Windows 10 2004+ (Build 19041+)

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
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
- **云端 API**：支持 OpenAI Whisper API、讯飞语音等，需填写 API Key

### 大模型 (LLM)

支持任意 OpenAI 兼容格式的 API：

| 服务商 | API URL |
|--------|---------|
| OpenAI | `https://api.openai.com/v1` |
| DeepSeek | `https://api.deepseek.com/v1` |
| 通义千问 | `https://dashscope.aliyuncs.com/compatible-mode/v1` |
| 智谱 | `https://open.bigmodel.cn/api/paas/v4` |
| Ollama (本地) | `http://localhost:11434` |

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

## 📄 开源协议

[MIT License](LICENSE)
