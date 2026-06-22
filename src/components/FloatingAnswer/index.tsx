import { useState } from "react";
import type { AnswerEntry } from "../../types";

const MOCK_ANSWERS: AnswerEntry[] = [];

export default function FloatingAnswer() {
  const [answers] = useState<AnswerEntry[]>(MOCK_ANSWERS);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const handleCopy = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 1500);
  };

  return (
    <div style={{
      height: "100%",
      overflow: "auto",
      padding: "12px",
      display: "flex",
      flexDirection: "column",
      gap: "12px",
    }}>
      {answers.length === 0 ? (
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
          <div style={{ fontSize: "13px" }}>等待音频输入...</div>
          <div style={{ fontSize: "11px", textAlign: "center", lineHeight: 1.6 }}>
            音频捕获已就绪<br />
            开始面试后，AI 将自动生成参考答案
          </div>
        </div>
      ) : (
        answers.map((item) => (
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
              <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>
                {item.latency_ms}ms
              </span>
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
        ))
      )}
    </div>
  );
}
