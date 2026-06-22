import { useState } from "react";
import FloatingAnswer from "./components/FloatingAnswer";
import Settings from "./components/Settings";
import { useConfig } from "./hooks/useConfig";

type View = "answer" | "settings";

function App() {
  const [view, setView] = useState<View>("answer");
  const { config, loading } = useConfig();

  if (loading) {
    return (
      <div style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        height: "100%",
        color: "var(--text-secondary)",
      }}>
        加载中...
      </div>
    );
  }

  return (
    <div style={{ width: "100%", height: "100%", display: "flex", flexDirection: "column" }}>
      {/* 顶部导航栏 */}
      <nav style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "6px 12px",
        background: "var(--bg-secondary)",
        borderBottom: "1px solid var(--border)",
        // 允许拖拽移动窗口
        ["WebkitAppRegion" as string]: "drag",
      }}>
        <div style={{ display: "flex", gap: "8px" }}>
          <button
            onClick={() => setView("answer")}
            style={{
              padding: "4px 12px",
              background: view === "answer" ? "var(--accent)" : "transparent",
              color: view === "answer" ? "#fff" : "var(--text-secondary)",
              ["WebkitAppRegion" as string]: "no-drag",
            }}
          >
            答案
          </button>
          <button
            onClick={() => setView("settings")}
            style={{
              padding: "4px 12px",
              background: view === "settings" ? "var(--accent)" : "transparent",
              color: view === "settings" ? "#fff" : "var(--text-secondary)",
              ["WebkitAppRegion" as string]: "no-drag",
            }}
          >
            设置
          </button>
        </div>
        <div style={{
          fontSize: "11px",
          color: "var(--text-muted)",
          ["WebkitAppRegion" as string]: "no-drag",
        }}>
          {config?.stt?.provider === "whisper-local" ? "本地" : "云端"} STT
        </div>
      </nav>

      {/* 内容区 */}
      <div style={{ flex: 1, overflow: "hidden" }}>
        {view === "answer" && <FloatingAnswer />}
        {view === "settings" && <Settings />}
      </div>
    </div>
  );
}

export default App;
