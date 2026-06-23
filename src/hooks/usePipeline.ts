import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { AnswerEntry, PipelineStatus } from "../types";

/// 管线事件类型（与 Rust 后端 PipelineEvent 对齐）
interface PipelineEvent {
  type: "Transcription" | "AnswerChunk" | "StateChange" | "Error";
  text?: string;
  confidence?: number;
  latency_ms?: number;
  content?: string;
  done?: boolean;
  state?: string;
  message?: string;
}

export function usePipeline() {
  const [status, setStatus] = useState<PipelineStatus>({
    running: false,
    audio_state: "stopped",
    stt_provider: "",
    llm_provider: "",
  });
  const [answers, setAnswers] = useState<AnswerEntry[]>([]);
  const [currentQuestion, setCurrentQuestion] = useState<string>("");
  const currentQuestionRef = useRef<string>("");
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 监听后端管线事件
  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    const setup = async () => {
      // 监听管线事件
      const unlisten = await listen<PipelineEvent>("pipeline-event", (event) => {
        const data = event.payload;

        switch (data.type) {
          case "Transcription":
            if (data.text) {
              setCurrentQuestion(data.text);
              currentQuestionRef.current = data.text;
              setIsGenerating(true);
            }
            break;

          case "AnswerChunk":
            if (data.done) {
              // 最后一个 chunk — 完成流式输出
              setAnswers((prev) => {
                const updated = [...prev];
                if (updated[0] && updated[0].isStreaming) {
                  updated[0] = {
                    ...updated[0],
                    answer: updated[0].answer + (data.content || ""),
                    isStreaming: false,
                  };
                }
                return updated;
              });
              setIsGenerating(false);
            } else if (data.content) {
              // 中间 chunk — 追加到当前流式答案
              setAnswers((prev) => {
                const updated = [...prev];
                if (updated[0] && updated[0].isStreaming) {
                  updated[0] = {
                    ...updated[0],
                    answer: updated[0].answer + data.content,
                  };
                } else {
                  // 开始新的流式答案
                  const newAnswer: AnswerEntry = {
                    id: Date.now().toString(),
                    question: currentQuestionRef.current || "(未知问题)",
                    answer: data.content,
                    timestamp: Date.now(),
                    latency_ms: 0,
                    isStreaming: true,
                  };
                  updated.unshift(newAnswer);
                }
                return updated;
              });
            }
            break;

          case "Error":
            setError(data.message || "未知错误");
            setIsGenerating(false);
            break;

          case "StateChange":
            // 状态变化由 get_pipeline_status 处理
            break;
        }
      });
      unlisteners.push(unlisten);

      // 监听全局快捷键事件
      const unlistenHotkey = await listen<{ action: string }>("hotkey", (event) => {
        const appWindow = getCurrentWindow();
        switch (event.payload.action) {
          case "toggle_visibility":
            appWindow.isVisible().then((visible) => {
              if (visible) {
                appWindow.hide();
              } else {
                appWindow.show();
                appWindow.setFocus();
              }
            });
            break;
          case "toggle_mute":
            toggleMute();
            break;
        }
      });
      unlisteners.push(unlistenHotkey);

      // 初始化时刷新状态（防止页面切换后状态丢失）
      try {
        const s = await invoke<PipelineStatus>("get_pipeline_status");
        setStatus(s);
      } catch (e) {
        console.error("初始化状态刷新失败:", e);
      }
    };

    setup();

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  // 启动管线
  const start = useCallback(async () => {
    try {
      setError(null);

      // 先检查当前状态，避免重复启动
      const currentStatus = await invoke<PipelineStatus>("get_pipeline_status");
      if (currentStatus.running) {
        // 管线已在运行，直接更新 UI 状态
        setStatus(currentStatus);
        return;
      }

      await invoke("start_pipeline");
      // 刷新状态
      const s = await invoke<PipelineStatus>("get_pipeline_status");
      setStatus(s);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  // 停止管线
  const stop = useCallback(async () => {
    try {
      await invoke("stop_pipeline");
      setStatus({
        running: false,
        audio_state: "stopped",
        stt_provider: "",
        llm_provider: "",
      });
    } catch (e) {
      setError(String(e));
    }
  }, []);

  // 切换静音
  const toggleMute = useCallback(async () => {
    try {
      await invoke("toggle_mute");
      const s = await invoke<PipelineStatus>("get_pipeline_status");
      setStatus(s);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  // 手动发送文本到 LLM（测试用）
  const sendText = useCallback(async (text: string) => {
    try {
      setError(null);
      setIsGenerating(true);
      setCurrentQuestion(text);

      const answer = await invoke<string>("process_audio_text", { text });

      const newAnswer: AnswerEntry = {
        id: Date.now().toString(),
        question: text,
        answer,
        timestamp: Date.now(),
        latency_ms: 0,
      };
      setAnswers((prev) => [newAnswer, ...prev]);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsGenerating(false);
    }
  }, []);

  // 下载 Whisper 模型
  const downloadModel = useCallback(async (modelName: string = "base") => {
    try {
      setError(null);
      const result = await invoke<string>("download_whisper_model", { modelName });
      return result;
    } catch (e) {
      setError(String(e));
      throw e;
    }
  }, []);

  // 获取模型状态
  const getModelStatus = useCallback(async (modelName: string = "base") => {
    try {
      return await invoke<{ model: string; exists: boolean; path: string; download_url: string }>(
        "get_whisper_model_status",
        { modelName }
      );
    } catch (e) {
      setError(String(e));
      throw e;
    }
  }, []);

  // 刷新状态
  const refreshStatus = useCallback(async () => {
    try {
      const s = await invoke<PipelineStatus>("get_pipeline_status");
      setStatus(s);
    } catch (e) {
      console.error("获取状态失败:", e);
    }
  }, []);

  // 清空答案
  const clearAnswers = useCallback(() => {
    setAnswers([]);
  }, []);

  return {
    status,
    answers,
    currentQuestion,
    isGenerating,
    error,
    start,
    stop,
    toggleMute,
    sendText,
    refreshStatus,
    clearAnswers,
    downloadModel,
    getModelStatus,
  };
}
