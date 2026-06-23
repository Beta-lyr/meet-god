import { useState, useEffect, useCallback } from "react";
import { useRecorder } from "../../hooks/useRecorder";
import SessionDetail from "./SessionDetail";
import {
  TrashIcon,
  DownloadIcon,
  ChevronDownIcon,
  HistoryIcon,
} from "../common/Icons";
import type { Session } from "../../types";

export default function SessionHistory() {
  const { sessions, loading, listSessions, deleteSession, exportSession } =
    useRecorder();
  const [selectedSession, setSelectedSession] = useState<string | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);
  const [exportMenu, setExportMenu] = useState<string | null>(null);

  useEffect(() => {
    listSessions();
  }, [listSessions]);

  // 点击外部关闭下拉菜单
  useEffect(() => {
    const handleClick = () => {
      setExportMenu(null);
      setDeleteConfirm(null);
    };
    if (exportMenu || deleteConfirm) {
      document.addEventListener("click", handleClick);
      return () => document.removeEventListener("click", handleClick);
    }
  }, [exportMenu, deleteConfirm]);

  const handleDelete = useCallback(
    async (sessionId: string) => {
      try {
        await deleteSession(sessionId);
        setDeleteConfirm(null);
      } catch {
        // 错误已在 hook 中处理
      }
    },
    [deleteSession]
  );

  const handleExport = useCallback(
    async (sessionId: string, format: "markdown" | "json") => {
      try {
        const content = await exportSession(sessionId, format);
        // 复制到剪贴板
        await navigator.clipboard.writeText(content);
        setExportMenu(null);
        // 可以添加一个 toast 提示
      } catch {
        // 错误已在 hook 中处理
      }
    },
    [exportSession]
  );

  const formatDate = (dateStr: string) => {
    try {
      const d = new Date(dateStr);
      return d.toLocaleString("zh-CN", {
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return dateStr;
    }
  };

  const getDisplayTitle = (session: Session) => {
    if (session.title && session.title.trim()) {
      return session.title;
    }
    // 用日期作为标题
    return `会话 ${formatDate(session.started_at)}`;
  };

  // 如果选中了某个会话，显示详情
  if (selectedSession) {
    return (
      <SessionDetail
        sessionId={selectedSession}
        onBack={() => setSelectedSession(null)}
      />
    );
  }

  return (
    <div className="settings-window">
      {/* Header */}
      <div className="settings-header">
        <div className="floating-header-left">
          <HistoryIcon size={14} />
          <span className="floating-title">会话记录</span>
        </div>
        <div className="floating-header-right">
          <span className="status-badge">{sessions.length} 条记录</span>
        </div>
      </div>

      {/* Content */}
      <div className="session-list-content">
        {loading && sessions.length === 0 ? (
          <div className="empty-state">
            <div className="loading-spinner" />
            <span className="empty-state-text">加载中...</span>
          </div>
        ) : sessions.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">
              <HistoryIcon size={36} />
            </div>
            <span className="empty-state-text">暂无会话记录</span>
            <span className="empty-state-hint">
              启动识别后，会话将自动记录
            </span>
          </div>
        ) : (
          <div className="session-list">
            {sessions.map((session) => (
              <div
                key={session.id}
                className="glass-card session-card"
                onClick={() => setSelectedSession(session.id)}
              >
                <div className="session-card-header">
                  <div className="session-card-info">
                    <span className="session-card-title">
                      {getDisplayTitle(session)}
                    </span>
                    <span className="session-card-meta">
                      {formatDate(session.started_at)}
                      {session.message_count > 0 &&
                        ` · ${session.message_count} 条消息`}
                    </span>
                  </div>
                  <div className="session-card-actions">
                    {/* Export button */}
                    <div className="session-dropdown-wrapper">
                      <button
                        className="btn-icon"
                        title="导出"
                        onClick={(e) => {
                          e.stopPropagation();
                          setExportMenu(
                            exportMenu === session.id ? null : session.id
                          );
                        }}
                      >
                        <DownloadIcon size={14} />
                      </button>
                      {exportMenu === session.id && (
                        <div className="session-dropdown">
                          <button
                            className="session-dropdown-item"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleExport(session.id, "markdown");
                            }}
                          >
                            导出 Markdown
                          </button>
                          <button
                            className="session-dropdown-item"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleExport(session.id, "json");
                            }}
                          >
                            导出 JSON
                          </button>
                        </div>
                      )}
                    </div>

                    {/* Delete button */}
                    {deleteConfirm === session.id ? (
                      <button
                        className="btn btn-danger"
                        style={{ fontSize: "var(--text-xs)", padding: "2px 8px" }}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDelete(session.id);
                        }}
                      >
                        确认删除
                      </button>
                    ) : (
                      <button
                        className="btn-icon"
                        title="删除"
                        onClick={(e) => {
                          e.stopPropagation();
                          setDeleteConfirm(session.id);
                        }}
                      >
                        <TrashIcon size={14} />
                      </button>
                    )}
                  </div>
                </div>
                {session.ended_at && (
                  <div className="session-card-duration">
                    {(() => {
                      try {
                        const start = new Date(session.started_at).getTime();
                        const end = new Date(session.ended_at).getTime();
                        const mins = Math.round((end - start) / 60000);
                        return mins > 0 ? `${mins} 分钟` : "不到 1 分钟";
                      } catch {
                        return "";
                      }
                    })()}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
