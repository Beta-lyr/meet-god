import { useState, useRef, useEffect } from "react";
import { usePipeline } from "../../hooks/usePipeline";
import {
  PlayIcon,
  PauseIcon,
  MicIcon,
  MicOffIcon,
  CopyIcon,
  CheckIcon,
  TrashIcon,
  SendIcon,
  DownloadIcon,
} from "../common/Icons";

export default function FloatingAnswer() {
  const {
    status,
    answers,
    isGenerating,
    error,
    start,
    stop,
    toggleMute,
    sendText,
    clearAnswers,
    downloadModel,
    getModelStatus,
  } = usePipeline();

  const [inputText, setInputText] = useState("");
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState("");
  const [modelReady, setModelReady] = useState<boolean | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Check model status on mount
  useEffect(() => {
    getModelStatus("base")
      .then((s) => setModelReady(s.exists))
      .catch(() => setModelReady(false));
  }, [getModelStatus]);

  const handleCopy = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 1500);
  };

  const handleDownload = async () => {
    setIsDownloading(true);
    setDownloadProgress("正在下载 Whisper base 模型 (~150MB)...");
    try {
      const result = await downloadModel("base");
      setDownloadProgress(result);
      setModelReady(true);
    } catch (e) {
      setDownloadProgress(`下载失败: ${e}`);
    } finally {
      setIsDownloading(false);
    }
  };

  const handleStart = async () => {
    if (modelReady === false) return;
    await start();
  };

  const handleSend = () => {
    if (inputText.trim()) {
      sendText(inputText.trim());
      setInputText("");
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const isRunning = status.running;
  const isMuted = status.audio_state === "muted";

  // Status dot class
  const statusDotClass = isRunning
    ? isMuted
      ? "status-dot processing"
      : "status-dot recording"
    : "status-dot idle";

  return (
    <div className="floating-window" style={{ width: "100%", height: "100%", resize: "none", borderRadius: 0 }}>
      {/* Header: control bar */}
      <div className="floating-header">
        <div className="floating-header-left">
          <div className={statusDotClass} />

          {/* Start / Stop */}
          <button
            className={`btn ${isRunning ? "btn-danger" : "btn-primary"}`}
            onClick={isRunning ? stop : handleStart}
            disabled={modelReady === false && !isRunning}
            style={{ fontSize: "var(--text-xs)", padding: "var(--space-3xs) var(--space-xs)" }}
          >
            {isRunning ? <PauseIcon size={10} /> : <PlayIcon size={10} />}
            {isRunning ? "停止" : "启动"}
          </button>

          {/* Mute toggle */}
          {isRunning && (
            <button
              className="btn btn-secondary"
              onClick={toggleMute}
              style={{ fontSize: "var(--text-xs)", padding: "var(--space-3xs) var(--space-xs)" }}
            >
              {isMuted ? <MicOffIcon size={11} /> : <MicIcon size={11} />}
              {isMuted ? "已静音" : "静音"}
            </button>
          )}
        </div>

        <div className="floating-header-right">
          {/* Clear */}
          {answers.length > 0 && (
            <button
              className="btn-icon"
              onClick={clearAnswers}
              title="清空会话记录"
            >
              <TrashIcon size={12} />
            </button>
          )}
        </div>
      </div>

      {/* Error Banner */}
      {error && (
        <div className="error-banner">
          {error}
        </div>
      )}

      {/* Model not ready prompt */}
      {modelReady === false && !isRunning && (
        <div className="download-prompt">
          <div className="download-prompt-icon">
            <DownloadIcon size={32} />
          </div>
          <div className="download-prompt-text">
            Whisper 语音识别模型尚未下载
          </div>
          <button
            className="btn btn-primary"
            onClick={handleDownload}
            disabled={isDownloading}
            style={{ padding: "var(--space-2xs) var(--space-lg)" }}
          >
            {isDownloading && <span className="loading-spinner" />}
            {isDownloading ? "下载中..." : "下载模型 (~150MB)"}
          </button>
          {downloadProgress && (
            <div style={{
              fontSize: "var(--text-sm)",
              color: downloadProgress.includes("失败") ? "var(--error)" : "var(--success)",
              textAlign: "center",
            }}>
              {downloadProgress}
            </div>
          )}
        </div>
      )}

      {/* Content: answer list */}
      <div className="floating-content">
        {answers.length === 0 && !isGenerating ? (
          <div className="empty-state">
            <div className="empty-state-icon">
              <MicIcon size={36} />
            </div>
            <div className="empty-state-text">
              {isRunning ? "等待音频输入..." : "点击「启动」开始"}
            </div>
            {isRunning && (
              <div className="empty-state-hint">
                STT: {status.stt_provider} | LLM: {status.llm_provider}
              </div>
            )}
          </div>
        ) : (
          <>
            {/* Generating indicator */}
            {isGenerating && (
              <div className="generating-indicator">
                <span className="loading-spinner" />
                正在生成答案...
              </div>
            )}

            {/* Answer cards */}
            {answers.map((item) => (
              <div key={item.id} className="card-answer">
                {/* Question label */}
                <div className="card-header">
                  <span className="card-label">
                    <MicIcon size={10} />
                    识别文本
                  </span>
                  {item.latency_ms > 0 && (
                    <span className="latency-tag">{item.latency_ms}ms</span>
                  )}
                </div>

                {/* Transcription text */}
                <div className="text-transcription" style={{ marginBottom: "var(--space-2xs)" }}>
                  {item.question}
                </div>

                {/* Answer divider */}
                <div className="card-header">
                  <span className="card-label">参考</span>
                </div>

                {/* Answer text */}
                <div className="text-answer">{item.answer}</div>

                {/* Actions */}
                <div className="card-actions">
                  <button
                    className="btn btn-secondary"
                    onClick={() => handleCopy(item.answer, item.id)}
                    style={{ fontSize: "var(--text-xs)", padding: "var(--space-4xs) var(--space-2xs)" }}
                  >
                    {copiedId === item.id ? <CheckIcon size={10} /> : <CopyIcon size={10} />}
                    {copiedId === item.id ? "已复制" : "复制"}
                  </button>
                </div>
              </div>
            ))}
          </>
        )}
      </div>

      {/* Footer: manual input */}
      <div className="floating-footer">
        <input
          ref={inputRef}
          type="text"
          className="floating-input"
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="输入文本，手动测试..."
        />
        <button
          className="btn btn-icon"
          onClick={handleSend}
          disabled={!inputText.trim()}
          style={{
            color: inputText.trim() ? "var(--accent)" : "var(--text-disabled)",
          }}
        >
          <SendIcon size={14} />
        </button>
      </div>
    </div>
  );
}
