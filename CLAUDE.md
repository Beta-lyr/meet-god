# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Meet-God is an AI interview copilot desktop application built with Tauri 2. It captures system audio in real-time, converts speech to text via Whisper, generates AI-powered reference answers, and displays them in a stealth floating window that is invisible to screen sharing/recording.

## Build & Dev Commands

```bash
# Frontend only (Vite dev server)
npm run dev

# Full Tauri dev (frontend + Rust backend)
npm run tauri dev

# With local Whisper STT (requires LLVM + CMake installed)
npm run tauri dev -- --features whisper-local

# TypeScript check
npx tsc --noEmit

# Rust check (without whisper)
cd src-tauri && cargo check

# Rust check (with whisper-local)
cd src-tauri && cargo check --features whisper-local

# Build release
npm run tauri build
```

## Environment Setup

- **Rust/Cargo**: Must be in PATH. After install, restart terminal or run `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`
- **LIBCLANG_PATH**: Required for whisper-local feature. Set to `C:\Program Files\LLVM\bin`
- **LLVM/Clang + CMake**: Only needed for whisper-local feature (optional)

## Architecture

### Two-process model (Tauri 2)

- **Rust backend** (`src-tauri/src/`): All heavy lifting — audio capture, STT, LLM API calls, stealth window control
- **React frontend** (`src/`): UI only — displays answers, settings, listens to backend events
- Communication: Tauri IPC commands (`#[tauri::command]`) + Tauri Events (`app.emit("pipeline-event", ...)`)

### Backend modules (`src-tauri/src/`)

| Module | Purpose |
|--------|---------|
| `audio/` | WASAPI Loopback audio capture via `cpal`. `AudioCaptureHandle` holds shared `Arc<Mutex<Vec<f32>>>` buffer. `cpal::Stream` is NOT Send — wrapped in `unsafe impl Send` `StreamWrapper` and leaked to a dedicated thread. |
| `stt/` | `SttProvider` trait. Two impls: `whisper_local` (local Whisper via whisper-rs, gated behind `whisper-local` feature) and `whisper_api` (OpenAI Whisper API). |
| `llm/` | `LlmProvider` trait. Two impls: `openai` (OpenAI-compatible `/v1/chat/completions`) and `anthropic` (Anthropic `/v1/messages`). |
| `pipeline/` | `PipelineEngine` orchestrates audio→VAD→STT→LLM. Runs in a background `tokio::spawn` loop, emits `PipelineEvent` via Tauri events to frontend. |
| `stealth/` | Windows `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` to hide window from screen capture. |
| `config/` | YAML config at `%APPDATA%/meet-god/config.yaml`. Schema defined in `config/schema.rs`. |

### Frontend (`src/`)

| File | Purpose |
|------|---------|
| `App.tsx` | Root: tab nav between answer view and settings |
| `components/FloatingAnswer/` | Main UI: pipeline control, answer display, model download |
| `components/Settings/` | Config panels for STT, LLM, profile |
| `hooks/usePipeline.ts` | Listens to `pipeline-event` Tauri events, manages start/stop/mute, exposes `downloadModel` and `getModelStatus` |
| `hooks/useConfig.ts` | Loads/saves config via Tauri commands |
| `types/index.ts` | TypeScript types mirroring Rust config schema |

### Key data flow

```
System Audio → WASAPI Loopback (cpal) → Audio Buffer (Arc<Mutex<Vec<f32>>>)
    → VAD → STT (whisper-local or API) → LLM (OpenAI or Anthropic format)
    → Tauri Event "pipeline-event" → Frontend renders answer
```

### Feature flags

- `whisper-local`: Enables local Whisper STT via whisper-rs. Requires LLVM/Clang. Default off.
- Without this flag, the app compiles and runs fine — only cloud STT providers are available.

## Important Patterns

- **Mutex types**: `tokio::sync::Mutex` for AppState fields held across `.await`; `std::sync::Mutex` for audio buffers (never held across await). Mixing them causes `Send` errors.
- **cpal::Stream is !Send**: Wrapped in `StreamWrapper(cpal::Stream)` with `unsafe impl Send`, moved to a dedicated thread via `std::thread::spawn`.
- **Tauri 2 window access**: Use `app.webview_windows()` in setup, or Tauri commands with `window: tauri::Window` parameter.
- **Config defaults**: Every config struct implements `Default` — the app works with zero config on first launch.
- **LLM providers**: Only two API formats — `openai` (compatible with DeepSeek, Qwen, Ollama, etc.) and `anthropic`. Users fill in their own URL/key/model.

## Platform

- Primary: **Windows 10 2004+** (WASAPI Loopback, `WDA_EXCLUDEFROMCAPTURE`)
- macOS/Linux: Not yet implemented (stealth and audio modules are Windows-specific)
