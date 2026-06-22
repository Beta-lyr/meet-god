import { useState } from "react";
import { useConfig } from "../../hooks/useConfig";
import type { AppConfig } from "../../types";

const LLM_FORMATS = [
  { value: "openai", label: "OpenAI 格式", desc: "兼容 /v1/chat/completions（适用于 OpenAI、DeepSeek、通义千问、智谱、Ollama 等）" },
  { value: "anthropic", label: "Anthropic 格式", desc: "Anthropic /v1/messages（Claude 系列）" },
];

type Tab = "stt" | "llm" | "profile";

export default function Settings() {
  const { config, saveConfig } = useConfig();
  const [tab, setTab] = useState<Tab>("stt");
  const [saving, setSaving] = useState(false);

  if (!config) return null;

  const handleSave = async (updates: Partial<AppConfig>) => {
    setSaving(true);
    const newConfig = { ...config, ...updates };
    await saveConfig(newConfig);
    setSaving(false);
  };

  return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column" }}>
      {/* Tab 栏 */}
      <div style={{
        display: "flex",
        borderBottom: "1px solid var(--border)",
        background: "var(--bg-secondary)",
      }}>
        {([["stt", "语音识别"], ["llm", "大模型"], ["profile", "资料"]] as [Tab, string][]).map(([key, label]) => (
          <button
            key={key}
            onClick={() => setTab(key)}
            style={{
              padding: "8px 16px",
              fontSize: "13px",
              background: "transparent",
              color: tab === key ? "var(--accent)" : "var(--text-secondary)",
              borderBottom: tab === key ? "2px solid var(--accent)" : "2px solid transparent",
              borderRadius: 0,
            }}
          >
            {label}
          </button>
        ))}
      </div>

      {/* 内容区 */}
      <div style={{ flex: 1, overflow: "auto", padding: "16px" }}>
        {tab === "stt" && (
          <SttSettings config={config} onSave={handleSave} saving={saving} />
        )}
        {tab === "llm" && (
          <LlmSettings config={config} onSave={handleSave} saving={saving} />
        )}
        {tab === "profile" && (
          <ProfileSettings config={config} onSave={handleSave} saving={saving} />
        )}
      </div>
    </div>
  );
}

function SttSettings({ config, onSave, saving }: { config: AppConfig; onSave: (u: Partial<AppConfig>) => Promise<void>; saving: boolean }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
      <Section title="语音识别模式">
        <Select
          label="Provider"
          value={config.stt.provider}
          options={[
            { value: "whisper-local", label: "本地 Whisper（推荐）" },
            { value: "openai", label: "OpenAI Whisper API" },
          ]}
          onChange={(v) => onSave({ stt: { ...config.stt, provider: v as AppConfig["stt"]["provider"] } })}
        />
      </Section>

      {config.stt.provider === "whisper-local" && (
        <Section title="本地 Whisper 配置">
          <Select
            label="模型大小"
            value={config.stt.local.model}
            options={[
              { value: "tiny", label: "tiny (~75MB, 最快)" },
              { value: "base", label: "base (~150MB, 推荐)" },
              { value: "small", label: "small (~500MB, 高精度)" },
              { value: "medium", label: "medium (~1.5GB, 最高精度)" },
            ]}
            onChange={(v) => onSave({ stt: { ...config.stt, local: { ...config.stt.local, model: v as AppConfig["stt"]["local"]["model"] } } })}
          />
          <Input
            label="语言"
            value={config.stt.local.language}
            placeholder="zh"
            onChange={(v) => onSave({ stt: { ...config.stt, local: { ...config.stt.local, language: v } } })}
          />
        </Section>
      )}

      {config.stt.provider === "openai" && (
        <Section title="API 配置">
          <Input
            label="API Key"
            value={config.stt.api.api_key}
            type="password"
            placeholder="sk-..."
            onChange={(v) => onSave({ stt: { ...config.stt, api: { ...config.stt.api, api_key: v } } })}
          />
          <Input
            label="API URL"
            value={config.stt.api.api_url}
            placeholder="https://api.openai.com/v1"
            onChange={(v) => onSave({ stt: { ...config.stt, api: { ...config.stt.api, api_url: v } } })}
          />
        </Section>
      )}
    </div>
  );
}

function LlmSettings({ config, onSave, saving }: { config: AppConfig; onSave: (u: Partial<AppConfig>) => Promise<void>; saving: boolean }) {
  const primary = config.llm.primary;
  const handlePrimary = (updates: Partial<AppConfig["llm"]["primary"]>) => {
    onSave({ llm: { ...config.llm, primary: { ...primary, ...updates } } });
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
      <Section title="主模型">
        <Select
          label="API 格式"
          value={primary.provider}
          options={LLM_FORMATS.map((f) => ({ value: f.value, label: f.label }))}
          onChange={(v) => handlePrimary({ provider: v as AppConfig["llm"]["primary"]["provider"] })}
        />
        <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "-6px" }}>
          {LLM_FORMATS.find((f) => f.value === primary.provider)?.desc}
        </div>
        <Input
          label="API URL"
          value={primary.api_url}
          placeholder={primary.provider === "anthropic" ? "https://api.anthropic.com" : "https://api.openai.com/v1"}
          onChange={(v) => handlePrimary({ api_url: v })}
        />
        <Input
          label="API Key"
          value={primary.api_key}
          type="password"
          placeholder="sk-..."
          onChange={(v) => handlePrimary({ api_key: v })}
        />
        <Input
          label="模型"
          value={primary.model}
          placeholder={primary.provider === "anthropic" ? "claude-sonnet-4-20250514" : "gpt-4o-mini"}
          onChange={(v) => handlePrimary({ model: v })}
        />
        <Input
          label="Temperature"
          value={String(primary.temperature)}
          type="number"
          onChange={(v) => handlePrimary({ temperature: parseFloat(v) || 0.7 })}
        />
      </Section>
    </div>
  );
}

function ProfileSettings({ config, onSave, saving }: { config: AppConfig; onSave: (u: Partial<AppConfig>) => Promise<void>; saving: boolean }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
      <Section title="个人资料">
        <Textarea
          label="简历内容"
          value={config.profile.resume}
          placeholder="粘贴你的简历内容，AI 将基于简历生成个性化答案..."
          rows={6}
          onChange={(v) => onSave({ profile: { ...config.profile, resume: v } })}
        />
        <Textarea
          label="目标岗位 JD"
          value={config.profile.job_description}
          placeholder="粘贴目标岗位的职位描述..."
          rows={4}
          onChange={(v) => onSave({ profile: { ...config.profile, job_description: v } })}
        />
        <Textarea
          label="自定义提示词"
          value={config.profile.custom_prompt}
          placeholder="可选，覆盖默认的 System Prompt..."
          rows={3}
          onChange={(v) => onSave({ profile: { ...config.profile, custom_prompt: v } })}
        />
      </Section>
    </div>
  );
}

// ========== 通用 UI 组件 ==========

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h3 style={{
        fontSize: "13px",
        color: "var(--text-secondary)",
        marginBottom: "12px",
        fontWeight: 500,
      }}>
        {title}
      </h3>
      <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
        {children}
      </div>
    </div>
  );
}

function Input({ label, value, type = "text", placeholder, onChange }: {
  label: string; value: string; type?: string; placeholder?: string; onChange: (v: string) => void;
}) {
  return (
    <div>
      <label style={{ fontSize: "12px", color: "var(--text-muted)", marginBottom: "4px", display: "block" }}>
        {label}
      </label>
      <input
        type={type}
        value={value}
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
        style={{ width: "100%", padding: "6px 10px", fontSize: "13px" }}
      />
    </div>
  );
}

function Select({ label, value, options, onChange }: {
  label: string; value: string; options: { value: string; label: string }[]; onChange: (v: string) => void;
}) {
  return (
    <div>
      <label style={{ fontSize: "12px", color: "var(--text-muted)", marginBottom: "4px", display: "block" }}>
        {label}
      </label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        style={{ width: "100%", padding: "6px 10px", fontSize: "13px" }}
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>{o.label}</option>
        ))}
      </select>
    </div>
  );
}

function Textarea({ label, value, placeholder, rows = 3, onChange }: {
  label: string; value: string; placeholder?: string; rows?: number; onChange: (v: string) => void;
}) {
  return (
    <div>
      <label style={{ fontSize: "12px", color: "var(--text-muted)", marginBottom: "4px", display: "block" }}>
        {label}
      </label>
      <textarea
        value={value}
        placeholder={placeholder}
        rows={rows}
        onChange={(e) => onChange(e.target.value)}
        style={{
          width: "100%",
          padding: "6px 10px",
          fontSize: "13px",
          resize: "vertical",
          minHeight: "60px",
        }}
      />
    </div>
  );
}
