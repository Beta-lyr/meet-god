import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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
              setIsGenerating(true);
            }
            break;

          case "AnswerChunk":
            if (data.content) {
              const newAnswer: AnswerEntry = {
                id: Date.now().toString(),
                question: currentQuestion || "(未知问题)",
                answer: data.content,
                timestamp: Date.now(),
                latency_ms: data.latency_ms || 0,
              };
              setAnswers((prev) => [newAnswer, ...prev]);
              setIsGenerating(false);
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
    };

    setup();

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  }, [currentQuestion]);

  // 启动管线
  const start = useCallback(async () => {
    try {
      setError(null);
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
  };
}
