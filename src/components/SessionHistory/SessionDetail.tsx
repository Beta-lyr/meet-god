import { useState, useEffect, useCallback, useRef } from "react";
import { useRecorder } from "../../hooks/useRecorder";
import {
  ArrowLeftIcon,
  DownloadIcon,
  BookmarkIcon,
  StarIcon,
  CopyIcon,
  CheckIcon,
} from "../common/Icons";
import type { SessionMessage } from "../../types";

interface SessionDetailProps {
  sessionId: string;
  onBack: () => void;
}

const BOOKMARK_OPTIONS: { value: string; label: string; icon: typeof StarIcon }[] = [
  { value: "important", label: "重要", icon: StarIcon },
  { value: "mastered", label: "已掌握", icon: CheckIcon },
  { value: "review", label: "待复习", icon: BookmarkIcon },
];

export default function SessionDetail({
  sessionId,
  onBack,
}: SessionDetailProps) {
  const { getSessionMessages, updateBookmark, exportSession } = useRecorder();
  const [messages, setMessages] = useState<SessionMessage[]>([]);
  const [loading, setLoading] = useState(true);
  const [copied, setCopied] = useState(false);
  const [exportMenuOpen, setExportMenuOpen] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);

  const loadMessages = useCallback(async () => {
    setLoading(true);
    const msgs = await getSessionMessages(sessionId);
    setMessages(msgs);
    setLoading(false);
  }, [sessionId, getSessionMessages]);

  useEffect(() => {
    loadMessages();
  }, [loadMessages]);

  // 点击外部关闭导出菜单
  useEffect(() => {
    if (exportMenuOpen) {
      const handleClick = () => setExportMenuOpen(false);
      document.addEventListener("click", handleClick);
      return () => document.removeEventListener("click", handleClick);
    }
  }, [exportMenuOpen]);

  const handleBookmark = useCallback(
    async (messageId: string, bookmark: string | null) => {
      await updateBookmark(messageId, bookmark);
      setMessages((prev) =>
        prev.map((m) =>
          m.id === messageId ? { ...m, bookmark } : m
        )
      );
    },
    [updateBookmark]
  );

  const handleExport = useCallback(
    async (format: "markdown" | "json") => {
      try {
        const content = await exportSession(sessionId, format);
        await navigator.clipboard.writeText(content);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
        setExportMenuOpen(false);
      } catch {
        // error handled in hook
      }
    },
    [sessionId, exportSession]
  );

  const formatTime = (dateStr: string) => {
    try {
      const d = new Date(dateStr);
      return d.toLocaleTimeString("zh-CN", {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      });
    } catch {
      return "";
    }
  };

  const getBookmarkLabel = (bm: string | null) => {
    switch (bm) {
      case "important":
        return "重要";
      case "mastered":
        return "已掌握";
      case "review":
        return "待复习";
      default:
        return null;
    }
  };

  // 计算会话时长
  const getSessionDuration = () => {
    if (messages.length < 2) return "";
    try {
      const first = new Date(messages[0].created_at).getTime();
      const last = new Date(messages[messages.length - 1].created_at).getTime();
      const mins = Math.round((last - first) / 60000);
      return mins > 0 ? `${mins} 分钟` : "不到 1 分钟";
    } catch {
      return "";
    }
  };

  return (
    <div className="settings-window">
      {/* Header */}
      <div className="settings-header">
        <div className="floating-header-left">
          <button
            className="btn-icon"
            onClick={onBack}
            title="返回列表"
          >
            <ArrowLeftIcon size={14} />
          </button>
          <span className="floating-title">会话详情</span>
        </div>
        <div className="floating-header-right">
          {getSessionDuration() && (
            <span className="status-badge">{getSessionDuration()}</span>
          )}
          <div className="session-dropdown-wrapper">
            <button
              className="btn-icon"
              title={copied ? "已复制" : "导出"}
              onClick={(e) => {
                e.stopPropagation();
                setExportMenuOpen(!exportMenuOpen);
              }}
            >
              {copied ? (
                <CheckIcon size={14} />
              ) : (
                <DownloadIcon size={14} />
              )}
            </button>
            {exportMenuOpen && (
              <div className="session-dropdown session-dropdown-right">
                <button
                  className="session-dropdown-item"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleExport("markdown");
                  }}
                >
                  复制 Markdown
                </button>
                <button
                  className="session-dropdown-item"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleExport("json");
                  }}
                >
                  复制 JSON
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Message List */}
      <div className="session-detail-content" ref={contentRef}>
        {loading ? (
          <div className="empty-state">
            <div className="loading-spinner" />
            <span className="empty-state-text">加载中...</span>
          </div>
        ) : messages.length === 0 ? (
          <div className="empty-state">
            <span className="empty-state-text">暂无消息记录</span>
          </div>
        ) : (
          <div className="session-messages">
            {messages.map((msg) => (
              <div
                key={msg.id}
                className={
                  msg.role === "question"
                    ? "card-transcription session-message"
                    : "card-answer session-message"
                }
              >
                <div className="card-header">
                  <span className="card-label">
                    {msg.role === "question" ? "问题" : "回答"}
                    <span className="session-msg-time">
                      {formatTime(msg.created_at)}
                    </span>
                    {msg.latency_ms > 0 && (
                      <span className="latency-tag">{msg.latency_ms}ms</span>
                    )}
                  </span>
                </div>
                <div
                  className={
                    msg.role === "question"
                      ? "text-transcription"
                      : "text-answer"
                  }
                >
                  {msg.content}
                </div>

                {/* Bookmark bar */}
                <div className="session-bookmark-bar">
                  {msg.bookmark && (
                    <span className={`session-bookmark-tag session-bookmark-${msg.bookmark}`}>
                      {getBookmarkLabel(msg.bookmark)}
                    </span>
                  )}
                  <div className="session-bookmark-buttons">
                    {BOOKMARK_OPTIONS.map((opt) => (
                      <button
                        key={opt.value}
                        className={`btn-pill ${
                          msg.bookmark === opt.value ? "active" : ""
                        }`}
                        onClick={() =>
                          handleBookmark(
                            msg.id,
                            msg.bookmark === opt.value ? null : opt.value
                          )
                        }
                        title={opt.label}
                      >
                        <opt.icon size={10} />
                        {opt.label}
                      </button>
                    ))}
                    {msg.bookmark && (
                      <button
                        className="btn-pill"
                        onClick={() => handleBookmark(msg.id, null)}
                        title="清除标签"
                      >
                        清除
                      </button>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
