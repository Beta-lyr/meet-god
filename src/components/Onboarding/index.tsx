import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { usePipeline } from "../../hooks/usePipeline";
import type { AppConfig } from "../../types";
import {
  EyeIcon,
  EyeOffIcon,
  CheckIcon,
  DownloadIcon,
  SettingsIcon,
} from "../common/Icons";

const TOTAL_STEPS = 4;

const LLM_FORMATS = [
  { value: "openai", label: "OpenAI" },
  { value: "anthropic", label: "Anthropic" },
] as const;

const WHISPER_MODELS = [
  { value: "tiny", label: "tiny", desc: "~75MB, 最快" },
  { value: "base", label: "base", desc: "~150MB, 推荐" },
  { value: "small", label: "small", desc: "~500MB, 高精度" },
  { value: "medium", label: "medium", desc: "~1.5GB, 最高精度" },
] as const;

interface OnboardingProps {
  config: AppConfig;
  onComplete: (config: AppConfig) => void;
}

export default function Onboarding({ config, onComplete }: OnboardingProps) {
  const [step, setStep] = useState(1);
  const [draft, setDraft] = useState<AppConfig>({ ...config });
  const [testResult, setTestResult] = useState<string | null>(null);
  const [testing, setTesting] = useState(false);
  const [modelExists, setModelExists] = useState<boolean | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState("");
  const { downloadModel, getModelStatus } = usePipeline();

  const updateDraft = useCallback(
    (updates: Partial<AppConfig>) => {
      setDraft((prev) => ({ ...prev, ...updates }));
    },
    []
  );

  const updateLlmPrimary = useCallback(
    (updates: Partial<AppConfig["llm"]["primary"]>) => {
      setDraft((prev) => ({
        ...prev,
        llm: { ...prev.llm, primary: { ...prev.llm.primary, ...updates } },
      }));
    },
    []
  );

  // Step 2: Test LLM connection
  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      // Save config temporarily to use process_audio_text
      await invoke("save_config", { newConfig: draft });
      const result = await invoke<string>("process_audio_text", {
        text: "你好，请简单介绍一下你自己。",
      });
      setTestResult(`连接成功: ${result.slice(0, 80)}...`);
    } catch (e) {
      setTestResult(`连接失败: ${e}`);
    } finally {
      setTesting(false);
    }
  };

  // Step 3: Check/download model
  const handleCheckModel = async () => {
    try {
      const status = await getModelStatus(draft.stt.local.model);
      setModelExists(status.exists);
    } catch {
      setModelExists(false);
    }
  };

  const handleDownloadModel = async () => {
    setDownloading(true);
    setDownloadProgress("正在下载...");
    try {
      const result = await downloadModel(draft.stt.local.model);
      setDownloadProgress(result);
      setModelExists(true);
    } catch (e) {
      setDownloadProgress(`下载失败: ${e}`);
    } finally {
      setDownloading(false);
    }
  };

  const handleNext = () => {
    if (step < TOTAL_STEPS) {
      setStep(step + 1);
    }
  };

  const handleBack = () => {
    if (step > 1) {
      setStep(step - 1);
    }
  };

  const handleComplete = async () => {
    await invoke("save_config", { newConfig: draft });
    onComplete(draft);
  };

  const handleSkip = () => {
    if (step < TOTAL_STEPS) {
      setStep(step + 1);
    }
  };

  return (
    <div className="onboarding-window">
      {/* Header */}
      <div className="onboarding-header">
        <div className="onboarding-logo">Meet God</div>
        <div className="onboarding-tagline">你的辅助工具</div>
      </div>

      {/* Step Indicator */}
      <div className="step-indicator">
        {[1, 2, 3, 4].map((s, i) => (
          <div key={s} style={{ display: "flex", alignItems: "center" }}>
            <div
              className={`step-dot${step === s ? " active" : ""}${
                step > s ? " completed" : ""
              }`}
            >
              <div className="step-dot-circle">
                {step > s ? <CheckIcon size={12} /> : s}
              </div>
              <span className="step-dot-label">
                {s === 1 && "欢迎"}
                {s === 2 && "模型配置"}
                {s === 3 && "语音设置"}
                {s === 4 && "完成"}
              </span>
            </div>
            {i < 3 && (
              <div className={`step-line${step > s ? " completed" : ""}`} />
            )}
          </div>
        ))}
      </div>

      {/* Content */}
      <div className="onboarding-content">
        {step === 1 && <StepWelcome />}
        {step === 2 && (
          <StepConfigureLLM
            draft={draft}
            onUpdateLlm={updateLlmPrimary}
            onTest={handleTestConnection}
            testing={testing}
            testResult={testResult}
          />
        )}
        {step === 3 && (
          <StepVoiceSetup
            draft={draft}
            onUpdateStt={(updates) =>
              updateDraft({
                stt: { ...draft.stt, ...updates },
              })
            }
            modelExists={modelExists}
            onCheckModel={handleCheckModel}
            onDownloadModel={handleDownloadModel}
            downloading={downloading}
            downloadProgress={downloadProgress}
          />
        )}
        {step === 4 && <StepDone draft={draft} />}
      </div>

      {/* Footer */}
      <div className="onboarding-footer">
        <div>
          {step > 1 && (
            <button className="btn btn-secondary" onClick={handleBack}>
              上一步
            </button>
          )}
        </div>
        <div style={{ display: "flex", gap: "var(--space-2xs)" }}>
          {step === 1 && (
            <button className="btn btn-primary" onClick={handleNext}>
              开始配置
            </button>
          )}
          {step === 2 && (
            <>
              <button className="btn btn-ghost" onClick={handleSkip}>
                跳过
              </button>
              <button className="btn btn-primary" onClick={handleNext}>
                下一步
              </button>
            </>
          )}
          {step === 3 && (
            <>
              <button className="btn btn-ghost" onClick={handleSkip}>
                跳过
              </button>
              <button className="btn btn-primary" onClick={handleNext}>
                下一步
              </button>
            </>
          )}
          {step === 4 && (
            <button className="btn btn-primary" onClick={handleComplete}>
              进入主界面
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

/* ============================================
   Step 1: Welcome
   ============================================ */

function StepWelcome() {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: "var(--space-lg)",
        textAlign: "center",
        padding: "var(--space-xl) 0",
      }}
    >
      <div
        style={{
          fontSize: "48px",
          lineHeight: 1,
        }}
      >
        <SettingsIcon size={48} className="" />
      </div>
      <div
        style={{
          fontSize: "var(--text-xl)",
          fontWeight: 600,
          color: "var(--text-primary)",
        }}
      >
        欢迎使用 Meet God
      </div>
      <div
        style={{
          fontSize: "var(--text-base)",
          color: "var(--text-tertiary)",
          maxWidth: "340px",
          lineHeight: 1.7,
        }}
      >
        智能辅助工具，实时识别音频内容并生成参考答案。
        <br />
        支持本地语音识别与多种大语言模型。
      </div>
      <div
        style={{
          display: "flex",
          gap: "var(--space-xs)",
          flexWrap: "wrap",
          justifyContent: "center",
        }}
      >
        {["实时语音识别", "智能答案生成", "屏幕捕获不可见"].map((feat) => (
          <span key={feat} className="glass-badge">
            {feat}
          </span>
        ))}
      </div>
    </div>
  );
}

/* ============================================
   Step 2: Configure LLM
   ============================================ */

function StepConfigureLLM({
  draft,
  onUpdateLlm,
  onTest,
  testing,
  testResult,
}: {
  draft: AppConfig;
  onUpdateLlm: (u: Partial<AppConfig["llm"]["primary"]>) => void;
  onTest: () => void;
  testing: boolean;
  testResult: string | null;
}) {
  const primary = draft.llm.primary;

  return (
    <div className="setting-group">
      <div className="glass-card">
        <div className="setting-group-title">大语言模型配置</div>

        {/* API Format */}
        <div className="setting-item">
          <span className="setting-label">API 格式</span>
          <div className="pill-group">
            {LLM_FORMATS.map((f) => (
              <button
                key={f.value}
                className={`btn-pill${primary.provider === f.value ? " active" : ""}`}
                onClick={() =>
                  onUpdateLlm({
                    provider: f.value as AppConfig["llm"]["primary"]["provider"],
                  })
                }
              >
                {f.label}
              </button>
            ))}
          </div>
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
            onChange={(e) => onUpdateLlm({ api_url: e.target.value })}
          />
        </div>

        {/* API Key */}
        <SecretInputField
          label="API Key"
          value={primary.api_key}
          placeholder="sk-..."
          onChange={(v) => onUpdateLlm({ api_key: v })}
        />

        {/* Model Name */}
        <div className="setting-item">
          <span className="setting-label">模型名称</span>
          <input
            className="input-text"
            type="text"
            value={primary.model}
            placeholder={
              primary.provider === "anthropic"
                ? "claude-sonnet-4-20250514"
                : "gpt-4o-mini"
            }
            onChange={(e) => onUpdateLlm({ model: e.target.value })}
          />
        </div>

        {/* Test Connection */}
        <div className="setting-item">
          <button
            className="btn btn-secondary"
            onClick={onTest}
            disabled={testing || !primary.api_key}
            style={{ alignSelf: "flex-start" }}
          >
            {testing && <span className="loading-spinner" />}
            {testing ? "测试中..." : "测试连接"}
          </button>
          {testResult && (
            <span
              className="setting-description"
              style={{
                color: testResult.includes("成功")
                  ? "var(--success)"
                  : "var(--error)",
              }}
            >
              {testResult}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

/* ============================================
   Step 3: Voice Setup
   ============================================ */

function StepVoiceSetup({
  draft,
  onUpdateStt,
  modelExists,
  onCheckModel,
  onDownloadModel,
  downloading,
  downloadProgress,
}: {
  draft: AppConfig;
  onUpdateStt: (u: Partial<AppConfig["stt"]>) => void;
  modelExists: boolean | null;
  onCheckModel: () => void;
  onDownloadModel: () => void;
  downloading: boolean;
  downloadProgress: string;
}) {
  return (
    <div className="setting-group">
      <div className="glass-card">
        <div className="setting-group-title">语音识别设置（可选）</div>

        {/* Model Size */}
        <div className="setting-item">
          <span className="setting-label">Whisper 模型大小</span>
          <div className="pill-group">
            {WHISPER_MODELS.map((m) => (
              <button
                key={m.value}
                className={`btn-pill${
                  draft.stt.local.model === m.value ? " active" : ""
                }`}
                onClick={() =>
                  onUpdateStt({
                    local: { ...draft.stt.local, model: m.value },
                  })
                }
              >
                {m.label}
              </button>
            ))}
          </div>
          <span className="setting-description">
            {WHISPER_MODELS.find((m) => m.value === draft.stt.local.model)?.desc}
          </span>
        </div>

        {/* Download */}
        <div className="setting-item">
          <div style={{ display: "flex", gap: "var(--space-2xs)", alignItems: "center" }}>
            <button
              className="btn btn-secondary"
              onClick={onCheckModel}
            >
              检查模型
            </button>
            {modelExists === true && (
              <span style={{ fontSize: "var(--text-sm)", color: "var(--success)" }}>
                <CheckIcon size={12} /> 模型已存在
              </span>
            )}
            {modelExists === false && (
              <button
                className="btn btn-primary"
                onClick={onDownloadModel}
                disabled={downloading}
              >
                {downloading && <span className="loading-spinner" />}
                <DownloadIcon size={12} />
                {downloading ? "下载中..." : "下载模型"}
              </button>
            )}
          </div>
          {downloadProgress && (
            <span
              className="setting-description"
              style={{
                color: downloadProgress.includes("失败")
                  ? "var(--error)"
                  : "var(--success)",
              }}
            >
              {downloadProgress}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

/* ============================================
   Step 4: Done
   ============================================ */

function StepDone({ draft }: { draft: AppConfig }) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: "var(--space-lg)",
        padding: "var(--space-lg) 0",
      }}
    >
      <div
        style={{
          width: "48px",
          height: "48px",
          borderRadius: "50%",
          background: "var(--success)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <CheckIcon size={24} className="" />
      </div>
      <div
        style={{
          fontSize: "var(--text-lg)",
          fontWeight: 600,
          color: "var(--text-primary)",
        }}
      >
        配置完成
      </div>

      <div className="glass-card" style={{ width: "100%", maxWidth: "360px" }}>
        <div className="setting-group-title">配置摘要</div>
        <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-2xs)" }}>
          <SummaryRow label="LLM 格式" value={draft.llm.primary.provider} />
          <SummaryRow label="LLM 模型" value={draft.llm.primary.model || "(未设置)"} />
          <SummaryRow label="API URL" value={draft.llm.primary.api_url || "(未设置)"} />
          <SummaryRow
            label="API Key"
            value={draft.llm.primary.api_key ? "********" : "(未设置)"}
          />
          <SummaryRow label="语音识别" value={draft.stt.provider === "whisper-local" ? "本地 Whisper" : "云端 API"} />
          {draft.stt.provider === "whisper-local" && (
            <SummaryRow label="Whisper 模型" value={draft.stt.local.model} />
          )}
        </div>
      </div>
    </div>
  );
}

function SummaryRow({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        fontSize: "var(--text-sm)",
      }}
    >
      <span style={{ color: "var(--text-tertiary)" }}>{label}</span>
      <span style={{ color: "var(--text-secondary)", fontFamily: "var(--font-mono)" }}>
        {value}
      </span>
    </div>
  );
}

/* ============================================
   Shared: Secret Input
   ============================================ */

function SecretInputField({
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
