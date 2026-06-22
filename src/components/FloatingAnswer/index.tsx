import { useState, useRef, useEffect } from "react";
import { usePipeline } from "../../hooks/usePipeline";

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

  // 检查模型状态
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
    if (modelReady === false) {
      // 模型不存在，提示下载
      return;
    }
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

  return (
    <div style={{
      height: "100%",
      display: "flex",
      flexDirection: "column",
    }}>
      {/* 控制栏 */}
      <div style={{
        display: "flex",
        alignItems: "center",
        gap: "8px",
        padding: "8px 12px",
        borderBottom: "1px solid var(--border)",
        background: "var(--bg-secondary)",
      }}>
        {/* 状态指示器 */}
        <div style={{
          width: "8px",
          height: "8px",
          borderRadius: "50%",
          background: isRunning
            ? (status.audio_state === "muted" ? "var(--warning)" : "var(--success)")
            : "var(--text-muted)",
        }} />

        {/* 启动/停止按钮 */}
        <button
          onClick={isRunning ? stop : handleStart}
          disabled={modelReady === false && !isRunning}
          style={{
            padding: "4px 12px",
            fontSize: "12px",
            background: isRunning
              ? "var(--error)"
              : modelReady === false
                ? "var(--bg-card)"
                : "var(--success)",
            color: modelReady === false && !isRunning ? "var(--text-muted)" : "#fff",
          }}
        >
          {isRunning ? "停止" : "启动"}
        </button>

        {/* 静音按钮 */}
        {isRunning && (
          <button
            onClick={toggleMute}
            style={{
              padding: "4px 12px",
              fontSize: "12px",
              background: status.audio_state === "muted" ? "var(--warning)" : "var(--bg-card)",
              color: status.audio_state === "muted" ? "#000" : "var(--text-secondary)",
              border: "1px solid var(--border)",
            }}
          >
            {status.audio_state === "muted" ? "已静音" : "静音"}
          </button>
        )}

        {/* 清空按钮 */}
        {answers.length > 0 && (
          <button
            onClick={clearAnswers}
            style={{
              padding: "4px 12px",
              fontSize: "12px",
              background: "transparent",
              color: "var(--text-muted)",
              border: "1px solid var(--border)",
              marginLeft: "auto",
            }}
          >
            清空
          </button>
        )}
      </div>

      {/* 错误提示 */}
      {error && (
        <div style={{
          padding: "8px 12px",
          background: "rgba(239, 68, 68, 0.15)",
          color: "var(--error)",
          fontSize: "12px",
          borderBottom: "1px solid var(--border)",
        }}>
          ⚠️ {error}
        </div>
      )}

      {/* 模型未就绪提示 */}
      {modelReady === false && !isRunning && (
        <div style={{
          padding: "16px",
          margin: "12px",
          background: "var(--bg-card)",
          borderRadius: "var(--radius)",
          border: "1px solid var(--warning)",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: "12px",
        }}>
          <div style={{ fontSize: "13px", color: "var(--text-secondary)", textAlign: "center" }}>
            Whisper 语音识别模型尚未下载
          </div>
          <button
            onClick={handleDownload}
            disabled={isDownloading}
            style={{
              padding: "8px 24px",
              fontSize: "13px",
              background: isDownloading ? "var(--bg-card)" : "var(--accent)",
              color: isDownloading ? "var(--text-muted)" : "#fff",
            }}
          >
            {isDownloading ? "下载中..." : "下载模型 (~150MB)"}
          </button>
          {downloadProgress && (
            <div style={{
              fontSize: "12px",
              color: downloadProgress.includes("失败") ? "var(--error)" : "var(--success)",
              textAlign: "center",
            }}>
              {downloadProgress}
            </div>
          )}
        </div>
      )}

      {/* 答案列表 */}
      <div style={{
        flex: 1,
        overflow: "auto",
        padding: "12px",
        display: "flex",
        flexDirection: "column",
        gap: "12px",
      }}>
        {answers.length === 0 && !isGenerating ? (
          <div style={{
            flex: 1,
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            gap: "12px",
            color: "var(--text-muted)",
          }}>
            <div style={{ fontSize: "32px", opacity: 0.5 }}>🎙️</div>
            <div style={{ fontSize: "13px" }}>
              {isRunning ? "等待音频输入..." : "点击「启动」开始"}
            </div>
            {isRunning && (
              <div style={{ fontSize: "11px", textAlign: "center", lineHeight: 1.6 }}>
                STT: {status.stt_provider} | LLM: {status.llm_provider}
              </div>
            )}
          </div>
        ) : (
          <>
            {/* 生成中提示 */}
            {isGenerating && (
              <div style={{
                padding: "12px",
                background: "var(--bg-card)",
                borderRadius: "var(--radius)",
                border: "1px solid var(--accent)",
                color: "var(--accent)",
                fontSize: "13px",
                textAlign: "center",
              }}>
                ⏳ 正在生成答案...
              </div>
            )}

            {/* 答案卡片 */}
            {answers.map((item) => (
              <div
                key={item.id}
                style={{
                  background: "var(--bg-card)",
                  borderRadius: "var(--radius)",
                  padding: "12px",
                  border: "1px solid var(--border)",
                }}
              >
                {/* 问题 */}
                <div style={{
                  fontSize: "12px",
                  color: "var(--text-secondary)",
                  marginBottom: "8px",
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                }}>
                  <span>🎤 {item.question}</span>
                  {item.latency_ms > 0 && (
                    <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>
                      {item.latency_ms}ms
                    </span>
                  )}
                </div>

                {/* 答案 */}
                <div style={{
                  fontSize: "13px",
                  lineHeight: 1.7,
                  color: "var(--text-primary)",
                  whiteSpace: "pre-wrap",
                }}>
                  {item.answer}
                </div>

                {/* 操作栏 */}
                <div style={{
                  marginTop: "8px",
                  display: "flex",
                  gap: "8px",
                  justifyContent: "flex-end",
                }}>
                  <button
                    onClick={() => handleCopy(item.answer, item.id)}
                    style={{
                      padding: "2px 8px",
                      fontSize: "11px",
                      background: copiedId === item.id ? "var(--success)" : "var(--bg-secondary)",
                      color: copiedId === item.id ? "#fff" : "var(--text-secondary)",
                      border: "1px solid var(--border)",
                    }}
                  >
                    {copiedId === item.id ? "已复制" : "复制"}
                  </button>
                </div>
              </div>
            ))}
          </>
        )}
      </div>

      {/* 手动输入栏 */}
      <div style={{
        display: "flex",
        gap: "8px",
        padding: "8px 12px",
        borderTop: "1px solid var(--border)",
        background: "var(--bg-secondary)",
      }}>
        <input
          ref={inputRef}
          type="text"
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="输入问题文本，手动测试 LLM..."
          style={{
            flex: 1,
            padding: "6px 10px",
            fontSize: "12px",
            background: "var(--bg-primary)",
          }}
        />
        <button
          onClick={handleSend}
          disabled={!inputText.trim()}
          style={{
            padding: "6px 16px",
            fontSize: "12px",
            background: inputText.trim() ? "var(--accent)" : "var(--bg-card)",
            color: inputText.trim() ? "#fff" : "var(--text-muted)",
          }}
        >
          发送
        </button>
      </div>
    </div>
  );
}
