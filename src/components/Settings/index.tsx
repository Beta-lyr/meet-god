import { useState } from "react";
import { useConfig } from "../../hooks/useConfig";
import type { AppConfig } from "../../types";
import {
  MicIcon,
  SettingsIcon,
  EyeIcon,
  EyeOffIcon,
} from "../common/Icons";

const LLM_FORMATS = [
  {
    value: "openai",
    label: "OpenAI 格式",
    desc: "兼容 /v1/chat/completions（适用于 OpenAI、DeepSeek、通义千问、智谱、Ollama 等）",
  },
  {
    value: "anthropic",
    label: "Anthropic 格式",
    desc: "Anthropic /v1/messages（Claude 系列）",
  },
];

type Tab = "stt" | "llm" | "profile";

const TAB_ITEMS: { key: Tab; label: string; icon: React.ReactNode }[] = [
  { key: "stt", label: "语音识别", icon: <MicIcon size={13} /> },
  { key: "llm", label: "大模型", icon: <SettingsIcon size={13} /> },
  { key: "profile", label: "个人资料", icon: <SettingsIcon size={13} /> },
];

export default function Settings() {
  const { config, saveConfig } = useConfig();
  const [tab, setTab] = useState<Tab>("stt");
  if (!config) return null;

  const handleSave = async (updates: Partial<AppConfig>) => {
    const newConfig = { ...config, ...updates };
    await saveConfig(newConfig);
  };

  return (
    <div className="settings-window">
      {/* Body: sidebar + content */}
      <div className="settings-body">
        {/* Sidebar */}
        <div className="settings-sidebar">
          <div className="tab-list-vertical">
            {TAB_ITEMS.map((item) => (
              <button
                key={item.key}
                className={`tab-item${tab === item.key ? " active" : ""}`}
                onClick={() => setTab(item.key)}
              >
                {item.icon}
                {item.label}
              </button>
            ))}
          </div>
        </div>

        {/* Content */}
        <div className="settings-content">
          {tab === "stt" && (
            <SttSettings config={config} onSave={handleSave} />
          )}
          {tab === "llm" && (
            <LlmSettings config={config} onSave={handleSave} />
          )}
          {tab === "profile" && (
            <ProfileSettings config={config} onSave={handleSave} />
          )}
        </div>
      </div>
    </div>
  );
}

/* ============================================
   STT Settings
   ============================================ */

function SttSettings({
  config,
  onSave,
}: {
  config: AppConfig;
  onSave: (u: Partial<AppConfig>) => Promise<void>;
}) {
  return (
    <div className="setting-group">
      {/* Provider Selection */}
      <div className="glass-card">
        <div className="setting-group-title">语音识别模式</div>
        <div className="setting-item">
          <span className="setting-label">Provider</span>
          <select
            className="input-select"
            value={config.stt.provider}
            onChange={(e) =>
              onSave({
                stt: {
                  ...config.stt,
                  provider: e.target.value as AppConfig["stt"]["provider"],
                },
              })
            }
          >
            <option value="whisper-local">本地 Whisper（推荐）</option>
            <option value="openai">OpenAI Whisper API</option>
          </select>
        </div>
      </div>

      {/* Whisper Local Config */}
      {config.stt.provider === "whisper-local" && (
        <div className="glass-card">
          <div className="setting-group-title">本地 Whisper 配置</div>

          <div className="setting-item">
            <span className="setting-label">模型大小</span>
            <div className="pill-group">
              {(["tiny", "base", "small", "medium"] as const).map((m) => (
                <button
                  key={m}
                  className={`btn-pill${config.stt.local.model === m ? " active" : ""}`}
                  onClick={() =>
                    onSave({
                      stt: {
                        ...config.stt,
                        local: { ...config.stt.local, model: m },
                      },
                    })
                  }
                >
                  {m}
                </button>
              ))}
            </div>
            <span className="setting-description">
              {config.stt.local.model === "tiny" && "~75MB, 最快"}
              {config.stt.local.model === "base" && "~150MB, 推荐"}
              {config.stt.local.model === "small" && "~500MB, 高精度"}
              {config.stt.local.model === "medium" && "~1.5GB, 最高精度"}
            </span>
          </div>

          <div className="setting-item">
            <span className="setting-label">语言</span>
            <input
              className="input-text"
              type="text"
              value={config.stt.local.language}
              placeholder="zh"
              onChange={(e) =>
                onSave({
                  stt: {
                    ...config.stt,
                    local: { ...config.stt.local, language: e.target.value },
                  },
                })
              }
            />
          </div>
        </div>
      )}

      {/* OpenAI API Config */}
      {config.stt.provider === "openai" && (
        <div className="glass-card">
          <div className="setting-group-title">API 配置</div>

          <SecretInput
            label="API Key"
            value={config.stt.api.api_key}
            placeholder="sk-..."
            onChange={(v) =>
              onSave({
                stt: {
                  ...config.stt,
                  api: { ...config.stt.api, api_key: v },
                },
              })
            }
          />

          <div className="setting-item">
            <span className="setting-label">API URL</span>
            <input
              className="input-text"
              type="text"
              value={config.stt.api.api_url}
              placeholder="https://api.openai.com/v1"
              onChange={(e) =>
                onSave({
                  stt: {
                    ...config.stt,
                    api: { ...config.stt.api, api_url: e.target.value },
                  },
                })
              }
            />
          </div>
        </div>
      )}
    </div>
  );
}

/* ============================================
   LLM Settings
   ============================================ */

function LlmSettings({
  config,
  onSave,
}: {
  config: AppConfig;
  onSave: (u: Partial<AppConfig>) => Promise<void>;
}) {
  const primary = config.llm.primary;
  const handlePrimary = (updates: Partial<AppConfig["llm"]["primary"]>) => {
    onSave({ llm: { ...config.llm, primary: { ...primary, ...updates } } });
  };

  return (
    <div className="setting-group">
      <div className="glass-card">
        <div className="setting-group-title">主模型</div>

        {/* API Format Pill Group */}
        <div className="setting-item">
          <span className="setting-label">API 格式</span>
          <div className="pill-group">
            {LLM_FORMATS.map((f) => (
              <button
                key={f.value}
                className={`btn-pill${primary.provider === f.value ? " active" : ""}`}
                onClick={() =>
                  handlePrimary({
                    provider: f.value as AppConfig["llm"]["primary"]["provider"],
                  })
                }
              >
                {f.label}
              </button>
            ))}
          </div>
          <span className="setting-description">
            {LLM_FORMATS.find((f) => f.value === primary.provider)?.desc}
          </span>
        </div>

        {/* API URL */}
        <div className="setting-item">
          <span className="setting-label">API URL</span>
          <input
            className="input-text"
            type="text"
            value={primary.api_url}
            placeholder={
              primary.provider === "anthropic"
                ? "https://api.anthropic.com"
                : "https://api.openai.com/v1"
            }
            onChange={(e) => handlePrimary({ api_url: e.target.value })}
          />
        </div>

        {/* API Key */}
        <SecretInput
          label="API Key"
          value={primary.api_key}
          placeholder="sk-..."
          onChange={(v) => handlePrimary({ api_key: v })}
        />

        {/* Model Name */}
        <div className="setting-item">
          <span className="setting-label">模型</span>
          <input
            className="input-text"
            type="text"
            value={primary.model}
            placeholder={
              primary.provider === "anthropic"
                ? "claude-sonnet-4-20250514"
                : "gpt-4o-mini"
            }
            onChange={(e) => handlePrimary({ model: e.target.value })}
          />
        </div>

        {/* Temperature */}
        <div className="setting-item">
          <span className="setting-label">Temperature</span>
          <input
            className="input-text"
            type="number"
            value={String(primary.temperature)}
            onChange={(e) =>
              handlePrimary({ temperature: parseFloat(e.target.value) || 0.7 })
            }
          />
        </div>
      </div>
    </div>
  );
}

/* ============================================
   Profile Settings
   ============================================ */

function ProfileSettings({
  config,
  onSave,
}: {
  config: AppConfig;
  onSave: (u: Partial<AppConfig>) => Promise<void>;
}) {
  return (
    <div className="setting-group">
      <div className="glass-card">
        <div className="setting-group-title">个人资料</div>

        <div className="setting-item">
          <span className="setting-label">简历内容</span>
          <textarea
            className="input-textarea"
            value={config.profile.resume}
            placeholder="粘贴你的简历内容，AI 将基于简历生成个性化答案..."
            rows={6}
            onChange={(e) =>
              onSave({ profile: { ...config.profile, resume: e.target.value } })
            }
          />
        </div>

        <div className="setting-item">
          <span className="setting-label">目标岗位 JD</span>
          <textarea
            className="input-textarea"
            value={config.profile.job_description}
            placeholder="粘贴目标岗位的职位描述..."
            rows={4}
            onChange={(e) =>
              onSave({
                profile: { ...config.profile, job_description: e.target.value },
              })
            }
          />
        </div>

        <div className="setting-item">
          <span className="setting-label">自定义提示词</span>
          <textarea
            className="input-textarea"
            value={config.profile.custom_prompt}
            placeholder="可选，覆盖默认的 System Prompt..."
            rows={3}
            onChange={(e) =>
              onSave({
                profile: { ...config.profile, custom_prompt: e.target.value },
              })
            }
          />
        </div>
      </div>
    </div>
  );
}

/* ============================================
   Shared: Secret Input with Eye Toggle
   ============================================ */

function SecretInput({
  label,
  value,
  placeholder,
  onChange,
}: {
  label: string;
  value: string;
  placeholder?: string;
  onChange: (v: string) => void;
}) {
  const [visible, setVisible] = useState(false);

  return (
    <div className="setting-item">
      <span className="setting-label">{label}</span>
      <div style={{ position: "relative" }}>
        <input
          className="input-secret"
          type={visible ? "text" : "password"}
          value={value}
          placeholder={placeholder}
          onChange={(e) => onChange(e.target.value)}
        />
        <button
          className="btn-icon"
          type="button"
          onClick={() => setVisible(!visible)}
          style={{
            position: "absolute",
            right: "4px",
            top: "50%",
            transform: "translateY(-50%)",
            width: "24px",
            height: "24px",
          }}
        >
          {visible ? <EyeOffIcon size={12} /> : <EyeIcon size={12} />}
        </button>
      </div>
    </div>
  );
}
