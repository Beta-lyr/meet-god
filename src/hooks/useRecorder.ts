import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Session, SessionMessage } from "../types";

export function useRecorder() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // 监听会话启动事件
  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    const setup = async () => {
      const unlisten = await listen<{ session_id: string }>(
        "session-started",
        (event) => {
          setCurrentSessionId(event.payload.session_id);
        }
      );
      unlisteners.push(unlisten);

      const unlistenState = await listen<{ state: string }>(
        "pipeline-event",
        (event) => {
          if (
            event.payload &&
            typeof event.payload === "object" &&
            "type" in event.payload &&
            (event.payload as { type: string }).type === "StateChange"
          ) {
            const state = (event.payload as { state?: string }).state;
            if (state === "stopped") {
              setCurrentSessionId(null);
            }
          }
        }
      );
      unlisteners.push(unlistenState);
    };

    setup();

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  // 列出会话
  const listSessions = useCallback(async (limit?: number) => {
    setLoading(true);
    try {
      const result = await invoke<Session[]>("list_sessions", {
        limit: limit ?? 50,
      });
      setSessions(result);
      return result;
    } catch (e) {
      console.error("获取会话列表失败:", e);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  // 获取会话消息
  const getSessionMessages = useCallback(
    async (sessionId: string): Promise<SessionMessage[]> => {
      try {
        return await invoke<SessionMessage[]>("get_session_messages", {
          sessionId,
        });
      } catch (e) {
        console.error("获取会话消息失败:", e);
        return [];
      }
    },
    []
  );

  // 更新书签
  const updateBookmark = useCallback(
    async (messageId: string, bookmark: string | null) => {
      try {
        await invoke("update_bookmark", { messageId, bookmark });
      } catch (e) {
        console.error("更新书签失败:", e);
      }
    },
    []
  );

  // 删除会话
  const deleteSession = useCallback(
    async (sessionId: string) => {
      try {
        await invoke("delete_session", { sessionId });
        setSessions((prev) => prev.filter((s) => s.id !== sessionId));
      } catch (e) {
        console.error("删除会话失败:", e);
        throw e;
      }
    },
    []
  );

  // 导出会话
  const exportSession = useCallback(
    async (sessionId: string, format: "markdown" | "json"): Promise<string> => {
      try {
        return await invoke<string>("export_session", {
          sessionId,
          format,
        });
      } catch (e) {
        console.error("导出会话失败:", e);
        throw e;
      }
    },
    []
  );

  return {
    sessions,
    currentSessionId,
    loading,
    listSessions,
    getSessionMessages,
    updateBookmark,
    deleteSession,
    exportSession,
  };
}
