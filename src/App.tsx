import { useState, useEffect } from "react";
import FloatingAnswer from "./components/FloatingAnswer";
import Settings from "./components/Settings";
import Onboarding from "./components/Onboarding";
import SessionHistory from "./components/SessionHistory";
import { useConfig } from "./hooks/useConfig";
import { SettingsIcon, HistoryIcon, MinimizeIcon, MaximizeIcon, CloseIcon } from "./components/common/Icons";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

type View = "answer" | "settings" | "history";

function App() {
  const [view, setView] = useState<View>("answer");
  const { config, loading, saveConfig } = useConfig();
  const [showOnboarding, setShowOnboarding] = useState(false);
  const appWindow = getCurrentWindow();

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
    <div className="floating-window">
      {/* Top Navigation Bar */}
      <nav className="floating-header">
        <div className="floating-header-left">
          <button
            className={`tab-btn ${view === "answer" ? "active" : ""}`}
            onClick={() => setView("answer")}
          >
            识别
          </button>
          <button
            className={`tab-btn ${view === "settings" ? "active" : ""}`}
            onClick={() => setView("settings")}
          >
            <SettingsIcon size={12} />
            设置
          </button>
          <button
            className={`tab-btn ${view === "history" ? "active" : ""}`}
            onClick={() => setView("history")}
          >
            <HistoryIcon size={12} />
            记录
          </button>
        </div>
        <div className="floating-header-right">
          <span className="status-badge">
            {config?.stt?.provider === "whisper-local" ? "本地" : "云端"} STT
          </span>
          <div className="window-controls">
            <button
              className="btn-icon window-btn"
              onClick={() => appWindow.minimize()}
              title="最小化"
            >
              <MinimizeIcon size={12} />
            </button>
            <button
              className="btn-icon window-btn"
              onClick={() => appWindow.toggleMaximize()}
              title="最大化"
            >
              <MaximizeIcon size={12} />
            </button>
            <button
              className="btn-icon window-btn close-btn"
              onClick={() => appWindow.hide()}
              title="隐藏到托盘"
            >
              <CloseIcon size={12} />
            </button>
          </div>
        </div>
      </nav>

      {/* Content Area */}
      <div className="floating-content-area">
        {view === "answer" && <FloatingAnswer />}
        {view === "settings" && <Settings />}
        {view === "history" && <SessionHistory />}
      </div>
    </div>
  );
}

export default App;
