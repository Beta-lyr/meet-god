import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "../types";

const DEFAULT_CONFIG: AppConfig = {
  audio: {
    device: "default",
    sample_rate: 16000,
    channels: 1,
    vad_enabled: true,
    vad_threshold: 0.5,
  },
  stt: {
    provider: "whisper-local",
    local: { model: "base", language: "zh", device: "cpu" },
    api: { api_key: "", api_url: "", model: "whisper-1", timeout_ms: 10000 },
  },
  llm: {
    primary: {
      provider: "openai",
      api_key: "",
      api_url: "https://api.openai.com/v1",
      model: "gpt-4o-mini",
      temperature: 0.7,
      max_tokens: 1024,
      timeout_ms: 15000,
    },
    fallback: {
      provider: "",
      api_key: "",
      api_url: "",
      model: "",
      temperature: 0.7,
      max_tokens: 1024,
      timeout_ms: 15000,
    },
  },
  ui: { opacity: 0.9, font_size: 14, always_on_top: true, theme: "dark" },
  hotkeys: {
    toggle_visibility: "Ctrl+Shift+H",
    toggle_mute: "Ctrl+Shift+M",
    emergency_exit: "Ctrl+Shift+Escape",
  },
  profile: { resume: "", job_description: "", custom_prompt: "" },
};

export function useConfig() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<AppConfig>("get_config")
      .then((c) => setConfig(c))
      .catch((e) => {
        console.error("加载配置失败:", e);
        setConfig(DEFAULT_CONFIG);
      })
      .finally(() => setLoading(false));
  }, []);

  const saveConfig = useCallback(async (newConfig: AppConfig) => {
    try {
      await invoke("save_config", { newConfig });
      setConfig(newConfig);
      return true;
    } catch (e) {
      console.error("保存配置失败:", e);
      return false;
    }
  }, []);

  return { config, loading, saveConfig };
}
