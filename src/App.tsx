import { useState, useEffect } from "react";
import FloatingAnswer from "./components/FloatingAnswer";
import Settings from "./components/Settings";
import Onboarding from "./components/Onboarding";
import { useConfig } from "./hooks/useConfig";
import { SettingsIcon } from "./components/common/Icons";
import { listen } from "@tauri-apps/api/event";

type View = "answer" | "settings";

function App() {
  const [view, setView] = useState<View>("answer");
  const { config, loading, saveConfig } = useConfig();
  const [showOnboarding, setShowOnboarding] = useState(false);

  // Check if onboarding is needed (no API key configured)
  useEffect(() => {
    if (!loading && config) {
      const hasApiKey =
        config.llm.primary.api_key && config.llm.primary.api_key.trim() !== "";
      setShowOnboarding(!hasApiKey);
    }
  }, [loading, config]);

  // Listen for navigate events from tray menu
  useEffect(() => {
    const setup = async () => {
      const unlisten = await listen<string>("navigate", (event) => {
        if (event.payload === "settings") {
          setView("settings");
        }
      });
      return unlisten;
    };
    let unlistenFn: (() => void) | undefined;
    setup().then((fn) => {
      unlistenFn = fn;
    });
    return () => {
      unlistenFn?.();
    };
  }, []);

  if (loading) {
    return (
      <div className="empty-state">
        <div className="loading-spinner" />
        <span className="empty-state-text">加载中...</span>
      </div>
    );
  }

  if (showOnboarding && config) {
    return (
      <Onboarding
        config={config}
        onComplete={(newConfig) => {
          saveConfig(newConfig);
          setShowOnboarding(false);
        }}
      />
    );
  }

  return (
    <div style={{ width: "100%", height: "100%", display: "flex", flexDirection: "column" }}>
      {/* Top Navigation Bar */}
      <nav className="floating-header">
        <div className="floating-header-left">
          <button
            className="btn btn-ghost"
            onClick={() => setView("answer")}
            style={{
              fontSize: "var(--text-sm)",
              padding: "var(--space-3xs) var(--space-xs)",
              color: view === "answer" ? "var(--accent-text)" : "var(--text-tertiary)",
              background: view === "answer" ? "var(--accent-subtle)" : "transparent",
            }}
          >
            识别
          </button>
          <button
            className="btn btn-ghost"
            onClick={() => setView("settings")}
            style={{
              fontSize: "var(--text-sm)",
              padding: "var(--space-3xs) var(--space-xs)",
              color: view === "settings" ? "var(--accent-text)" : "var(--text-tertiary)",
              background: view === "settings" ? "var(--accent-subtle)" : "transparent",
            }}
          >
            <SettingsIcon size={12} />
            设置
          </button>
        </div>
        <div className="floating-header-right">
          <span className="status-badge">
            {config?.stt?.provider === "whisper-local" ? "本地" : "云端"} STT
          </span>
        </div>
      </nav>

      {/* Content Area */}
      <div style={{ flex: 1, overflow: "hidden" }}>
        {view === "answer" && <FloatingAnswer />}
        {view === "settings" && <Settings />}
      </div>
    </div>
  );
}

export default App;
