# Meet-God 前端 UI 设计规范

> 版本：v1.0
> 日期：2026-06-23
> 风格：Glassmorphism（玻璃拟态）
> 配色：中性灰 + 青色点缀
> 模式：仅深色模式
> 技术约束：React 19 + TypeScript + 原生 CSS 变量（无 UI 库）

---

## 一、设计理念

### 1.1 风格定位

**Glassmorphism（玻璃拟态）** — 通过半透明背景、backdrop-blur 毛玻璃效果和微光边框，营造层次分明 yet 沉浸的视觉体验。

选择理由：
- 悬浮窗场景天然适合玻璃质感，在任意应用背景上都有融入感
- 半透明特性与窗口透明度调节功能天然契合
- 深色玻璃在面试场景中不抢眼，降低存在感
- 比纯扁平设计更有层次，比拟物风格更现代

### 1.2 设计原则

1. **信息层级清晰** — 识别文本 > 参考内容 > 控制元素，视觉权重递减
2. **呼吸感留白** — 内容不拥挤，卡片间保持充足负空间
3. **克制动效** — 微妙的过渡动画，不分散注意力
4. **隐蔽优先** — 界面中不出现"面试"、"AI"、"Copilot"等敏感词汇
5. **原生实现** — 所有效果用纯 CSS 变量 + 类名实现，不依赖外部 UI 库

### 1.3 隐蔽命名约定

界面文案使用隐晦命名，避免敏感词汇：

| 原义 | 界面显示 | 说明 |
|------|---------|------|
| 面试官问题 | 识别文本 | STT 识别结果 |
| AI 答案 | 参考 | 参考内容 |
| 面试记录 | 会话记录 | 历史会话 |
| 面试辅助 | Meet God | 仅应用名，不出现在功能文案中 |

---

## 二、色彩系统

### 2.1 CSS 变量定义

所有色彩通过 CSS 自定义属性（CSS Custom Properties）定义在 `:root` 中，全局统一引用。

```css
:root {
  /* ====== 基底背景 ====== */
  --bg-base: #0a0a0f;              /* 最底层背景，近乎纯黑带微蓝 */
  --bg-elevated: #12121a;          /* 次级背景，窗口底色 */
  --bg-surface: #1a1a24;          /* 卡片/面板底色 */

  /* ====== 玻璃表面（核心） ====== */
  --glass-bg: rgba(255, 255, 255, 0.04);       /* 玻璃面板背景 */
  --glass-bg-hover: rgba(255, 255, 255, 0.07); /* 悬停态 */
  --glass-bg-active: rgba(255, 255, 255, 0.10);/* 按下态 */
  --glass-border: rgba(255, 255, 255, 0.08);  /* 玻璃边框 */
  --glass-border-strong: rgba(255, 255, 255, 0.14); /* 强调边框 */
  --glass-highlight: rgba(255, 255, 255, 0.06); /* 顶部高光（inset shadow） */

  /* ====== 文字 ====== */
  --text-primary: #e4e4e7;         /* 主文本 */
  --text-secondary: #a1a1aa;       /* 次要文本 */
  --text-tertiary: #71717a;         /* 辅助/占位文本 */
  --text-disabled: #52525b;         /* 禁用态 */
  --text-inverse: #0a0a0f;         /* 反色文本（青色背景上） */

  /* ====== 青色强调（唯一 accent） ====== */
  --accent: #06b6d4;                /* 主强调色 cyan-500 */
  --accent-hover: #22d3ee;         /* 悬停 cyan-400 */
  --accent-active: #0891b2;        /* 按下 cyan-600 */
  --accent-subtle: rgba(6, 182, 212, 0.12);  /* 强调背景 */
  --accent-glow: rgba(6, 182, 212, 0.25);    /* 辉光效果 */

  /* ====== 功能色 ====== */
  --color-success: #10b981;         /* 成功/已连接 */
  --color-warning: #f59e0b;         /* 警告/处理中 */
  --color-error: #ef4444;           /* 错误/录制中 */
  --color-info: #06b6d4;            /* 信息（同 accent） */

  /* ====== 功能色半透明背景 ====== */
  --success-bg: rgba(16, 185, 129, 0.12);
  --warning-bg: rgba(245, 158, 11, 0.12);
  --error-bg: rgba(239, 68, 68, 0.12);

  /* ====== 边框 ====== */
  --border-subtle: rgba(255, 255, 255, 0.06);
  --border-default: rgba(255, 255, 255, 0.10);
  --border-strong: rgba(255, 255, 255, 0.16);

  /* ====== 阴影 ====== */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.5);
  --shadow-glass: 0 8px 32px rgba(0, 0, 0, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.05);
  --shadow-glow: 0 0 20px rgba(6, 182, 212, 0.15);
  --shadow-glow-strong: 0 0 24px rgba(6, 182, 212, 0.30);
}
```

### 2.2 色彩使用规则

| 用途 | 变量 | 说明 |
|------|------|------|
| 窗口背景 | `--bg-elevated` | 悬浮窗/设置窗底色 |
| 卡片/面板 | `--glass-bg` + backdrop-blur | 玻璃质感面板 |
| 主文本 | `--text-primary` | 识别文本、答案内容 |
| 次要文本 | `--text-secondary` | 标签、时间戳 |
| 占位符 | `--text-tertiary` | input placeholder |
| 强调按钮 | `--accent` | 主操作按钮 |
| 录制状态 | `--color-error` + 脉冲动画 | 管线运行中 |
| 处理状态 | `--color-warning` + 脉冲动画 | STT/LLM 处理中 |
| 空闲状态 | `--text-tertiary` | 管线未启动 |

---

## 三、字体系统

### 3.1 字体栈

```css
:root {
  --font-sans: 'Segoe UI', 'Microsoft YaHei', system-ui, -apple-system, sans-serif;
  --font-mono: 'Cascadia Code', 'Consolas', 'Courier New', monospace;
}
```

选择 Segoe UI 作为主字体：Windows 系统原生字体，无需额外加载，渲染清晰，与系统风格一致。中英文混排时使用 Microsoft YaHei 作为 fallback。

### 3.2 字号层级

```css
:root {
  --text-xs: 11px;     /* 标签、时间戳、辅助信息 */
  --text-sm: 12px;     /* 次要文本、设置项描述 */
  --text-base: 13px;   /* 正文（悬浮窗紧凑布局） */
  --text-md: 14px;     /* 主要内容、答案文本 */
  --text-lg: 16px;     /* 小标题、设置项标题 */
  --text-xl: 18px;     /* 区块标题 */
  --text-2xl: 22px;    /* 引导页标题 */
}
```

### 3.3 字重与行高

```css
:root {
  --font-normal: 400;    /* 正文 */
  --font-medium: 500;    /* 按钮、标签 */
  --font-semibold: 600;  /* 小标题 */
  --font-bold: 700;      /* 强调 */

  --leading-tight: 1.3;   /* 标题 */
  --leading-normal: 1.5;  /* 正文 */
  --leading-relaxed: 1.7; /* 长文本（答案内容） */
}
```

### 3.4 文字样式类

```css
/* 识别文本（STT 结果） */
.text-transcription {
  font-size: var(--text-md);
  font-weight: var(--font-normal);
  line-height: var(--leading-relaxed);
  color: var(--text-secondary);
}

/* 参考内容（AI 答案） */
.text-answer {
  font-size: var(--text-md);
  font-weight: var(--font-normal);
  line-height: var(--leading-relaxed);
  color: var(--text-primary);
}

/* 标签 */
.text-label {
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  line-height: var(--leading-tight);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* 区块标题 */
.text-section-title {
  font-size: var(--text-lg);
  font-weight: var(--font-semibold);
  line-height: var(--leading-tight);
  color: var(--text-primary);
}
```

---

## 四、间距与圆角

### 4.1 间距系统（4px 基准）

```css
:root {
  --space-4xs: 2px;
  --space-3xs: 4px;
  --space-2xs: 8px;
  --space-xs: 12px;
  --space-sm: 16px;
  --space-md: 20px;
  --space-lg: 24px;
  --space-xl: 32px;
  --space-2xl: 40px;
  --space-3xl: 48px;
}
```

使用规则：
- 组件内部 padding：`--space-sm`（16px）为默认
- 组件间距 gap：`--space-xs`（12px）为默认
- 区块间距 margin：`--space-lg`（24px）为默认
- 紧凑区域（悬浮窗顶部栏）：`--space-2xs`（8px）

### 4.2 圆角系统

```css
:root {
  --radius-sm: 6px;     /* 小按钮、标签、输入框 */
  --radius-md: 10px;    /* 卡片、下拉菜单 */
  --radius-lg: 14px;    /* 面板、设置区块 */
  --radius-xl: 20px;    /* 悬浮窗主体 */
  --radius-full: 9999px;/* 胶囊按钮、圆形图标 */
}
```

---

## 五、玻璃拟态效果规范

### 5.1 核心玻璃面板

```css
.glass-panel {
  background: var(--glass-bg);
  backdrop-filter: blur(16px) saturate(180%);
  -webkit-backdrop-filter: blur(16px) saturate(180%);
  border: 1px solid var(--glass-border);
  box-shadow: var(--shadow-glass);
  border-radius: var(--radius-xl);
}

/* 悬浮窗主体 — 更强的玻璃效果 */
.glass-window {
  background: rgba(18, 18, 26, 0.85);
  backdrop-filter: blur(20px) saturate(160%);
  -webkit-backdrop-filter: blur(20px) saturate(160%);
  border: 1px solid var(--glass-border);
  box-shadow: var(--shadow-lg), inset 0 1px 0 var(--glass-highlight);
  border-radius: var(--radius-xl);
  overflow: hidden;
}
```

### 5.2 玻璃卡片（内容区）

```css
.glass-card {
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-md);
  padding: var(--space-sm);
  transition: background 0.2s ease, border-color 0.2s ease;
}

.glass-card:hover {
  background: var(--glass-bg-hover);
  border-color: var(--glass-border-strong);
}
```

### 5.3 玻璃徽章/标签

```css
.glass-badge {
  background: var(--glass-bg);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-full);
  padding: var(--space-3xs) var(--space-2xs);
  font-size: var(--text-xs);
  color: var(--text-secondary);
}
```

### 5.4 玻璃输入控件

```css
.glass-input {
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: var(--space-2xs) var(--space-xs);
  color: var(--text-primary);
  font-size: var(--text-base);
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.glass-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-subtle);
}

.glass-input::placeholder {
  color: var(--text-tertiary);
}
```

---

## 六、组件库

### 6.1 按钮

```css
/* ===== 基础按钮类 ===== */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2xs);
  padding: var(--space-2xs) var(--space-sm);
  border-radius: var(--radius-sm);
  font-size: var(--text-base);
  font-weight: var(--font-medium);
  cursor: pointer;
  transition: all 0.15s ease;
  border: none;
  white-space: nowrap;
  user-select: none;
}

/* ===== 主要按钮（青色强调） ===== */
.btn-primary {
  background: var(--accent);
  color: var(--text-inverse);
  font-weight: var(--font-semibold);
}
.btn-primary:hover {
  background: var(--accent-hover);
  box-shadow: var(--shadow-glow);
}
.btn-primary:active {
  background: var(--accent-active);
  transform: scale(0.97);
}
.btn-primary:disabled {
  background: var(--text-disabled);
  cursor: not-allowed;
  box-shadow: none;
}

/* ===== 次要按钮（玻璃质感） ===== */
.btn-secondary {
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid var(--glass-border);
  color: var(--text-primary);
}
.btn-secondary:hover {
  background: var(--glass-bg-hover);
  border-color: var(--glass-border-strong);
}

/* ===== 幽灵按钮（仅悬停显示背景） ===== */
.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
}
.btn-ghost:hover {
  background: var(--glass-bg-hover);
  color: var(--text-primary);
}

/* ===== 危险按钮 ===== */
.btn-danger {
  background: var(--error-bg);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: var(--color-error);
}
.btn-danger:hover {
  background: rgba(239, 68, 68, 0.2);
  border-color: rgba(239, 68, 68, 0.5);
}

/* ===== 图标按钮（方形） ===== */
.btn-icon {
  width: 32px;
  height: 32px;
  padding: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s ease;
  border: none;
}
.btn-icon:hover {
  background: var(--glass-bg-hover);
  color: var(--text-primary);
}
.btn-icon.active {
  background: var(--accent-subtle);
  color: var(--accent);
}

/* ===== 胶囊按钮（标签/筛选） ===== */
.btn-pill {
  padding: var(--space-3xs) var(--space-xs);
  border-radius: var(--radius-full);
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  cursor: pointer;
  transition: all 0.15s ease;
}
.btn-pill.active {
  background: var(--accent-subtle);
  border-color: var(--accent);
  color: var(--accent);
}
```

### 6.2 卡片

```css
/* ===== 识别文本卡片 ===== */
.card-transcription {
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-md);
  padding: var(--space-sm);
  border-left: 3px solid var(--text-tertiary);  /* 左侧色条标识 */
  transition: border-color 0.3s ease;
}

/* 实时识别中 — 左侧色条变为青色 */
.card-transcription.active {
  border-left-color: var(--accent);
}

/* ===== 参考内容卡片 ===== */
.card-answer {
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-md);
  padding: var(--space-sm);
  border-left: 3px solid var(--accent);  /* 青色色条 */
  box-shadow: var(--shadow-sm);
}

/* 流式输出中 — 微弱辉光 */
.card-answer.streaming {
  border-color: var(--glass-border-strong);
  box-shadow: var(--shadow-glow);
}

/* ===== 卡片头部（标签 + 操作） ===== */
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-xs);
}

.card-label {
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: var(--space-2xs);
}
```

### 6.3 输入控件

```css
/* ===== 文本输入框 ===== */
.input-text {
  width: 100%;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: var(--space-2xs) var(--space-xs);
  color: var(--text-primary);
  font-size: var(--text-base);
  font-family: var(--font-sans);
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}
.input-text:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-subtle);
}
.input-text::placeholder {
  color: var(--text-tertiary);
}

/* ===== 多行文本框（设置页简历/JD） ===== */
.input-textarea {
  width: 100%;
  min-height: 120px;
  resize: vertical;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: var(--space-xs);
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-family: var(--font-sans);
  line-height: var(--leading-normal);
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}
.input-textarea:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-subtle);
}

/* ===== 密码输入框（API Key） ===== */
.input-secret {
  width: 100%;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: var(--space-2xs) var(--space-xs);
  color: var(--text-primary);
  font-size: var(--text-base);
  font-family: var(--font-mono);  /* 等宽字体 */
  letter-spacing: 0.05em;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}
.input-secret:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-subtle);
}

/* ===== 下拉选择框 ===== */
.input-select {
  width: 100%;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: var(--space-2xs) var(--space-xs);
  color: var(--text-primary);
  font-size: var(--text-base);
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%2371717a' stroke-width='2'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right var(--space-xs) center;
  padding-right: var(--space-xl);
  transition: border-color 0.2s ease;
}
.input-select:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-subtle);
}
```

### 6.4 滑块（透明度调节）

```css
/* ===== 自定义滑块 ===== */
.slider {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 4px;
  background: var(--border-default);
  border-radius: var(--radius-full);
  outline: none;
  cursor: pointer;
}

/* 滑块轨道（已选中部分） */
.slider::-webkit-slider-runnable-track {
  height: 4px;
  border-radius: var(--radius-full);
}

/* 滑块手柄 */
.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--accent);
  border: 2px solid var(--bg-elevated);
  cursor: pointer;
  margin-top: -5px;
  transition: box-shadow 0.2s ease;
}
.slider::-webkit-slider-thumb:hover {
  box-shadow: var(--shadow-glow-strong);
}

/* Firefox 兼容 */
.slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--accent);
  border: 2px solid var(--bg-elevated);
  cursor: pointer;
}
```

### 6.5 开关（Toggle Switch）

```css
/* ===== 开关组件 ===== */
.toggle {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--border-default);
  border-radius: var(--radius-full);
  transition: background 0.2s ease;
}

.toggle-slider::before {
  content: "";
  position: absolute;
  height: 14px;
  width: 14px;
  left: 3px;
  bottom: 3px;
  background: var(--text-secondary);
  border-radius: 50%;
  transition: transform 0.2s ease, background 0.2s ease;
}

/* 选中态 */
.toggle input:checked + .toggle-slider {
  background: var(--accent);
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(16px);
  background: var(--text-inverse);
}

/* 禁用态 */
.toggle input:disabled + .toggle-slider {
  opacity: 0.5;
  cursor: not-allowed;
}
```

### 6.6 标签页（Settings 面板）

```css
/* ===== 垂直标签栏（设置页侧边） ===== */
.tab-list-vertical {
  display: flex;
  flex-direction: column;
  gap: var(--space-3xs);
  padding: var(--space-sm);
  border-right: 1px solid var(--border-subtle);
}

.tab-item {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-xs) var(--space-sm);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: var(--text-base);
  cursor: pointer;
  transition: all 0.15s ease;
  border: none;
  background: transparent;
  text-align: left;
  width: 100%;
}

.tab-item:hover {
  background: var(--glass-bg-hover);
  color: var(--text-primary);
}

.tab-item.active {
  background: var(--accent-subtle);
  color: var(--accent);
  font-weight: var(--font-medium);
}
```

### 6.7 状态指示器

```css
/* ===== 状态点（脉冲动画） ===== */
.status-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-tertiary);
  flex-shrink: 0;
}

.status-dot.recording {
  background: var(--color-error);
  animation: pulse-error 1.5s ease-in-out infinite;
}

.status-dot.processing {
  background: var(--color-warning);
  animation: pulse-warning 1.5s ease-in-out infinite;
}

.status-dot.active {
  background: var(--color-success);
}

.status-dot.idle {
  background: var(--text-tertiary);
}

/* ===== 状态标签（点 + 文字） ===== */
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2xs);
  padding: var(--space-3xs) var(--space-xs);
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
}

.status-badge.recording {
  background: var(--error-bg);
  border-color: rgba(239, 68, 68, 0.2);
  color: var(--color-error);
}

.status-badge.processing {
  background: var(--warning-bg);
  border-color: rgba(245, 158, 11, 0.2);
  color: var(--color-warning);
}

.status-badge.active {
  background: var(--success-bg);
  border-color: rgba(16, 185, 129, 0.2);
  color: var(--color-success);
}

/* 脉冲动画 */
@keyframes pulse-error {
  0%, 100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4); }
  50% { box-shadow: 0 0 0 6px rgba(239, 68, 68, 0); }
}
@keyframes pulse-warning {
  0%, 100% { box-shadow: 0 0 0 0 rgba(245, 158, 11, 0.4); }
  50% { box-shadow: 0 0 0 6px rgba(245, 158, 11, 0); }
}
```

### 6.8 进度条（模型下载）

```css
/* ===== 下载进度条 ===== */
.progress-container {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: var(--space-2xs);
}

.progress-bar {
  width: 100%;
  height: 6px;
  background: var(--border-default);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent), var(--accent-hover));
  border-radius: var(--radius-full);
  transition: width 0.3s ease;
  box-shadow: var(--shadow-glow);
}

.progress-text {
  display: flex;
  justify-content: space-between;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
```

### 6.9 延迟显示标签

```css
/* ===== 延迟标签（答案卡片右下角） ===== */
.latency-tag {
  display: inline-flex;
  align-items: center;
  gap: var(--space-3xs);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;  /* 等宽数字 */
}

.latency-tag.good { color: var(--color-success); }
.latency-tag.ok { color: var(--color-warning); }
.latency-tag.slow { color: var(--color-error); }
```

---

## 七、界面设计

### 7.1 悬浮答案窗（主界面）

**窗口规格：** 默认 380px × 520px，可调整大小，始终置顶

**布局结构：**

```
┌──────────────────────────────────────────┐
│  ●idle  Meet God          [−] [□] [×]   │  顶部栏 40px
├──────────────────────────────────────────┤
│                                          │
│  识别文本                                │  标签
│  ┌──────────────────────────────────────┐ │
│  │ 请介绍一下你在上一家公司做的项目...  │ │  转录卡片
│  │                                      │ │
│  └──────────────────────────────────────┘ │
│                                          │
│  参考                          3.2s  ⎘   │  标签+延迟+复制
│  ┌──────────────────────────────────────┐ │
│  │ 在上一家公司，我负责了一个分布式...   │ │  答案卡片
│  │ ▌ (流式光标)                         │ │  (streaming)
│  │                                      │ │
│  │                                      │ │
│  │                                      │ │
│  └──────────────────────────────────────┘ │
│                                          │
│  识别文本                                │
│  ┌──────────────────────────────────────┐ │
│  │ 你在项目中遇到了什么技术挑战？       │ │  下一轮转录
│  └──────────────────────────────────────┘ │
│                                          │
├──────────────────────────────────────────┤
│  [输入测试...]                      [→]  │  底部输入 44px
└──────────────────────────────────────────┘
```

**顶部控制栏 CSS：**

```css
.floating-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 var(--space-sm);
  background: rgba(0, 0, 0, 0.15);
  border-bottom: 1px solid var(--border-subtle);
  -webkit-app-region: drag;  /* Tauri 拖拽区域 */
}

.floating-header-left {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}

.floating-header-right {
  display: flex;
  align-items: center;
  gap: var(--space-3xs);
  -webkit-app-region: no-drag;  /* 按钮可点击 */
}

.floating-title {
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  color: var(--text-secondary);
}
```

**顶部控制栏功能：**

| 位置 | 元素 | 说明 |
|------|------|------|
| 左侧 | 状态点 + 应用名 | 状态点反映管线状态（idle/recording/processing） |
| 右侧 | 启动/停止按钮 | 图标按钮，active 时青色 |
| 右侧 | 静音按钮 | 图标按钮，active 时显示斜杠 |
| 右侧 | 清空按钮 | 图标按钮，清除历史记录 |
| 右侧 | 透明度滑块 | 迷你滑块，悬停时展开 |

**内容滚动区 CSS：**

```css
.floating-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-sm);
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  scroll-behavior: smooth;
}

/* 自定义滚动条 */
.floating-content::-webkit-scrollbar {
  width: 4px;
}
.floating-content::-webkit-scrollbar-track {
  background: transparent;
}
.floating-content::-webkit-scrollbar-thumb {
  background: var(--border-default);
  border-radius: var(--radius-full);
}
.floating-content::-webkit-scrollbar-thumb:hover {
  background: var(--border-strong);
}
```

**底部输入栏 CSS：**

```css
.floating-footer {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  height: 44px;
  padding: 0 var(--space-sm);
  background: rgba(0, 0, 0, 0.15);
  border-top: 1px solid var(--border-subtle);
}

.floating-input {
  flex: 1;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: var(--space-2xs) var(--space-xs);
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.floating-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-subtle);
}
```

**模型未下载时的下载提示：**

```css
.download-prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-xl);
  text-align: center;
}

.download-prompt-icon {
  width: 48px;
  height: 48px;
  border-radius: var(--radius-lg);
  background: var(--accent-subtle);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
}

.download-prompt-text {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  line-height: var(--leading-normal);
}
```

**流式渲染光标动画：**

```css
/* 打字机光标 */
.streaming-cursor {
  display: inline-block;
  width: 2px;
  height: 1em;
  background: var(--accent);
  margin-left: 2px;
  vertical-align: text-bottom;
  animation: blink 0.8s steps(2) infinite;
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}
```

**透明度迷你滑块（顶部栏悬停展开）：**

```css
.opacity-control {
  display: flex;
  align-items: center;
  gap: var(--space-2xs);
  opacity: 0;
  width: 0;
  overflow: hidden;
  transition: opacity 0.2s ease, width 0.2s ease;
}

.floating-header:hover .opacity-control {
  opacity: 1;
  width: 80px;
}
```

**整体窗口结构：**

```css
.floating-window {
  width: 380px;
  height: 520px;
  min-width: 280px;
  min-height: 360px;
  display: flex;
  flex-direction: column;
  background: rgba(18, 18, 26, 0.85);
  backdrop-filter: blur(20px) saturate(160%);
  -webkit-backdrop-filter: blur(20px) saturate(160%);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-lg), inset 0 1px 0 var(--glass-highlight);
  overflow: hidden;
  resize: both;
}
```

---

### 7.2 设置面板

**窗口规格：** 640px × 480px，独立窗口

**布局结构：**

```
┌────────────────────────────────────────────────────┐
│  设置                                         [×]  │  顶栏 40px
├──────────────┬─────────────────────────────────────┤
│              │                                     │
│  🔧 语音识别  │  语音识别                           │  内容标题
│  🤖 大模型    │                                     │
│  👤 个人资料  │  Provider                           │  设置项
│              │  ┌───────────────────────────────┐  │
│              │  │ Whisper 本地 (推荐)      ▼  │  │
│              │  └───────────────────────────────┘  │
│              │                                     │
│              │  模型大小                            │
│              │  [tiny] [base] [small] [medium]     │  胶囊按钮组
│              │                                     │
│              │  语言                                │
│              │  ┌───────────────────────────────┐  │
│              │  │ 中文                       ▼  │  │
│              │  └───────────────────────────────┘  │
│              │                                     │
│              │  ───────────────────────────────     │
│              │                                     │
│              │  云端 API（可选）                    │  分组标题
│              │                                     │
│              │  API Key                            │
│              │  ┌───────────────────────────────┐  │
│              │  │ sk-•••••••••••••••       [👁] │  │  密码输入
│              │  └───────────────────────────────┘  │
│              │                                     │
│              │  API URL                            │
│              │  ┌───────────────────────────────┐  │
│              │  │ https://api.openai.com/v1     │  │
│              │  └───────────────────────────────┘  │
│              │                                     │
│              │           [取消]  [保存]             │  底部按钮
│              │                                     │
└──────────────┴─────────────────────────────────────┘
```

**设置面板整体结构 CSS：**

```css
.settings-window {
  width: 640px;
  height: 480px;
  display: flex;
  flex-direction: column;
  background: var(--bg-elevated);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 var(--space-sm);
  border-bottom: 1px solid var(--border-subtle);
  -webkit-app-region: drag;
}

.settings-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.settings-sidebar {
  width: 160px;
  border-right: 1px solid var(--border-subtle);
  padding: var(--space-sm) var(--space-xs);
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-lg);
}

.settings-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-xs);
  padding: var(--space-sm) var(--space-lg);
  border-top: 1px solid var(--border-subtle);
}
```

**设置项通用样式 CSS：**

```css
/* 单个设置项 */
.setting-item {
  display: flex;
  flex-direction: column;
  gap: var(--space-2xs);
  margin-bottom: var(--space-lg);
}

.setting-label {
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  color: var(--text-primary);
}

.setting-description {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  line-height: var(--leading-normal);
}

/* 设置项分组 */
.setting-group {
  margin-bottom: var(--space-xl);
}

.setting-group-title {
  font-size: var(--text-xs);
  font-weight: var(--font-semibold);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--space-sm);
  padding-bottom: var(--space-2xs);
  border-bottom: 1px solid var(--border-subtle);
}

/* 胶囊按钮组（模型大小选择等） */
.pill-group {
  display: flex;
  gap: var(--space-2xs);
}

.pill-group .btn-pill {
  flex: 1;
  text-align: center;
}
```

**三个 Tab 内容说明：**

**Tab 1 — 语音识别：**
- Provider 下拉选择（Whisper 本地 / OpenAI API / 讯飞 API）
- 本地模型大小：胶囊按钮组（tiny / base / small / medium）
- 语言选择下拉（中文 / English / 自动检测）
- 云端 API 配置（API Key + API URL），仅 provider 选云端时显示
- 模型状态提示（已下载 / 未下载 + 下载按钮）

**Tab 2 — 大模型：**
- API 格式选择（OpenAI 兼容 / Anthropic）胶囊按钮组
- API URL 输入框
- API Key 密码输入框（带显示/隐藏切换）
- 模型名称输入框
- Temperature 滑块（0.0 - 2.0，步长 0.1）
- 超时时间输入框
- 连接测试按钮 + 测试结果反馈

**Tab 3 — 个人资料：**
- 简历内容 Textarea（大文本框，支持 Markdown）
- 目标岗位 JD Textarea
- 自定义 System Prompt Textarea
- 字数统计显示

---

### 7.3 首次启动引导页

**窗口规格：** 480px × 420px，居中显示，独立窗口

**布局结构：**

```
┌──────────────────────────────────────────┐
│                                          │
│                                          │
│              ╔══════════╗                │  Logo 区
│              ║ Meet God ║                │
│              ╚══════════╝                │
│                                          │
│         你的面试辅助工具                  │  副标题
│        实时识别 · 智能参考                │
│                                          │
│    ●━━━━━━━━○──────────────             │  步骤指示器
│    1 配置模型  2 语音设置  3 完成          │
│                                          │
│    ─────────────────────────────────     │
│                                          │
│    API 格式                               │  内容区
│    [OpenAI 兼容]  [Anthropic]            │
│                                          │
│    API URL                               │
│    ┌──────────────────────────────────┐  │
│    │ https://api.openai.com/v1       │  │
│    └──────────────────────────────────┘  │
│                                          │
│    API Key                               │
│    ┌──────────────────────────────────┐  │
│    │ sk-••••••••••••          [👁]   │  │
│    └──────────────────────────────────┘  │
│                                          │
│    模型名称                               │
│    ┌──────────────────────────────────┐  │
│    │ gpt-4o-mini                      │  │
│    └──────────────────────────────────┘  │
│                                          │
│              [跳过]  [下一步 →]           │  导航按钮
│                                          │
└──────────────────────────────────────────┘
```

**引导页 CSS：**

```css
.onboarding-window {
  width: 480px;
  height: 420px;
  display: flex;
  flex-direction: column;
  background: var(--bg-elevated);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}

.onboarding-header {
  text-align: center;
  padding: var(--space-2xl) var(--space-xl) var(--space-lg);
}

.onboarding-logo {
  font-size: var(--text-2xl);
  font-weight: var(--font-bold);
  color: var(--text-primary);
  letter-spacing: 0.02em;
}

.onboarding-tagline {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  margin-top: var(--space-2xs);
}

/* 步骤指示器 */
.step-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-xs);
  padding: var(--space-sm) var(--space-xl);
}

.step-dot {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3xs);
}

.step-dot-circle {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--border-default);
  transition: background 0.3s ease;
}

.step-dot.completed .step-dot-circle {
  background: var(--accent);
}

.step-dot.current .step-dot-circle {
  background: var(--accent);
  box-shadow: var(--shadow-glow-strong);
}

.step-dot-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.step-dot.current .step-dot-label {
  color: var(--accent);
  font-weight: var(--font-medium);
}

.step-line {
  width: 40px;
  height: 2px;
  background: var(--border-default);
  border-radius: var(--radius-full);
  transition: background 0.3s ease;
}

.step-line.completed {
  background: var(--accent);
}

/* 内容区 */
.onboarding-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-lg) var(--space-xl);
}

/* 底部导航 */
.onboarding-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-sm) var(--space-xl);
  border-top: 1px solid var(--border-subtle);
}
```

**引导页步骤：**

1. **欢迎介绍** — Logo + 副标题 + 功能简介
2. **配置 LLM** — API 格式选择 + URL + Key + 模型名 + 测试连接按钮
3. **语音设置（可选跳过）** — 提示下载 Whisper 模型 + 模型大小选择 + 下载进度
4. **完成** — 配置摘要 + "进入主界面" 按钮

---

### 7.4 系统托盘菜单

托盘菜单为 Tauri 原生菜单组件，使用系统原生样式。仅需定义菜单项和图标。

**菜单项设计：**

```
┌──────────────────────┐
│  ● Meet God           │  标题（显示状态）
├──────────────────────┤
│  显示主窗口           │
│  隐藏主窗口           │
├──────────────────────┤
│  ▶ 开始               │
│  ⏸ 停止               │
│  🔇 静音              │
├──────────────────────┤
│  ⚙ 设置               │
├──────────────────────┤
│  退出                 │
└──────────────────────┘
```

**托盘图标状态：**

| 状态 | 图标样式 | 说明 |
|------|---------|------|
| 空闲 | 灰色 Logo | 管线未启动 |
| 运行中 | 青色 Logo | 管线运行中 |
| 静音 | 灰色 Logo + 斜杠 | 音频捕获暂停 |
| 错误 | 红色 Logo | 管线异常 |

---

## 八、动效规范

### 8.1 过渡动画

```css
:root {
  --duration-fast: 0.15s;    /* 按钮悬停、颜色变化 */
  --duration-normal: 0.2s;   /* 面板展开、滑动 */
  --duration-slow: 0.3s;     /* 页面切换、进度条 */
  --ease-out: cubic-bezier(0.16, 1, 0.3, 1);      /* 减速出 */
  --ease-in-out: cubic-bezier(0.65, 0, 0.35, 1);  /* 先加速后减速 */
}
```

### 8.2 关键动画

```css
/* ===== 答案卡片入场 ===== */
@keyframes card-enter {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.card-answer {
  animation: card-enter 0.3s var(--ease-out);
}

/* ===== 识别文本更新高亮 ===== */
@keyframes text-highlight {
  0% {
    background: var(--accent-subtle);
  }
  100% {
    background: transparent;
  }
}

.text-transcription.updated {
  animation: text-highlight 0.6s ease-out;
}

/* ===== 流式光标闪烁 ===== */
@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

/* ===== 状态点脉冲 ===== */
@keyframes pulse-error {
  0%, 100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4); }
  50% { box-shadow: 0 0 0 6px rgba(239, 68, 68, 0); }
}

@keyframes pulse-warning {
  0%, 100% { box-shadow: 0 0 0 0 rgba(245, 158, 11, 0.4); }
  50% { box-shadow: 0 0 0 6px rgba(245, 158, 11, 0); }
}

/* ===== 加载旋转 ===== */
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-default);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

/* ===== 渐入（引导页步骤切换） ===== */
@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateX(12px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.onboarding-content {
  animation: fade-in 0.3s var(--ease-out);
}
```

### 8.3 交互反馈

| 交互 | 动效 | 时长 |
|------|------|------|
| 按钮悬停 | 背景色变化 | 0.15s |
| 按钮按下 | scale(0.97) | 0.1s |
| 卡片入场 | 上滑+渐入 | 0.3s |
| 答案流式输出 | 光标闪烁 | 0.8s 循环 |
| 录制状态 | 红点脉冲 | 1.5s 循环 |
| 处理状态 | 橙点脉冲 | 1.5s 循环 |
| 步骤切换 | 横向渐入 | 0.3s |
| 滚动到新内容 | smooth scroll | 浏览器默认 |

---

## 九、响应式与适配

### 9.1 悬浮窗尺寸适配

```css
/* 默认尺寸 */
.floating-window {
  width: 380px;
  height: 520px;
}

/* 用户缩小到最小尺寸时 */
.floating-window:min-width {
  width: 280px;
  height: 360px;
}

/* 超小尺寸下隐藏部分元素 */
@media (max-width: 300px) {
  .floating-title { display: none; }
  .opacity-control { display: none; }
}

/* 内容区随窗口大小自适应 */
.floating-content {
  flex: 1;
  overflow-y: auto;
}
```

### 9.2 设置面板适配

```css
/* 默认尺寸 */
.settings-window {
  width: 640px;
  height: 480px;
}

/* 小屏幕下侧边栏收窄 */
@media (max-width: 500px) {
  .settings-sidebar {
    width: 120px;
  }
}
```

---

## 十、实现指南

### 10.1 CSS 文件组织

```
src/styles/
├── variables.css          # 所有 CSS 变量定义（色彩、字体、间距等）
├── base.css               # reset + 基础元素样式
├── components.css         # 通用组件样式（按钮、卡片、输入等）
├── animations.css         # 所有 @keyframes 动画定义
├── floating-window.css    # 悬浮答案窗专用样式
├── settings.css           # 设置面板专用样式
└── onboarding.css         # 引导页专用样式
```

### 10.2 变量引用方式

```css
/* 在 variables.css 中集中定义 */
:root {
  --accent: #06b6d4;
  --glass-bg: rgba(255, 255, 255, 0.04);
  /* ... */
}

/* 在各组件 CSS 中引用 */
.btn-primary {
  background: var(--accent);
  color: var(--text-inverse);
}
```

### 10.3 React 组件中的样式应用

```tsx
// 方式一：直接 className 引用（推荐）
import '../styles/floating-window.css';

function FloatingWindow() {
  return (
    <div className="floating-window glass-window">
      <div className="floating-header">
        <div className="floating-header-left">
          <span className={`status-dot ${status}`} />
          <span className="floating-title">Meet God</span>
        </div>
        <div className="floating-header-right">
          <button className="btn-icon" onClick={togglePipeline}>
            {isActive ? <PauseIcon /> : <PlayIcon />}
          </button>
        </div>
      </div>
      <div className="floating-content">
        {/* 内容 */}
      </div>
      <div className="floating-footer">
        <input className="floating-input" placeholder="输入测试..." />
      </div>
    </div>
  );
}
```

### 10.4 从 inline style 迁移建议

当前代码使用 inline style，建议迁移到 CSS 类名方式：

```tsx
// ❌ 之前 — inline style
<div style={{
  background: 'rgba(255, 255, 255, 0.04)',
  backdropFilter: 'blur(16px)',
  border: '1px solid rgba(255, 255, 255, 0.08)',
  borderRadius: '10px',
  padding: '16px',
}}>

// ✅ 之后 — CSS 类名
<div className="glass-card">
```

迁移步骤：
1. 先创建 `variables.css`，定义所有 CSS 变量
2. 创建 `components.css`，定义通用组件类
3. 逐个组件迁移 inline style → className
4. 为每个界面创建专用 CSS 文件
5. 在 `main.tsx` 中统一 import

### 10.5 Tauri 窗口透明度说明

窗口级别的透明度通过 Tauri API 控制，与 CSS opacity 不同：

```tsx
// Tauri 设置窗口透明度（整个窗口包括玻璃效果）
import { getCurrentWindow } from '@tauri-apps/api/window';
const appWindow = getCurrentWindow();
await appWindow.setOpacity(0.9);  // 0.0 - 1.0
```

注意：`setOpacity` 影响整个窗口的透明度，与 CSS `opacity` 不同。玻璃效果中的 `rgba` 半透明是视觉层面的，不受 `setOpacity` 影响。

### 10.6 拖拽实现

```tsx
// Tauri 窗口拖拽 — 在 CSS 中标记拖拽区域
// .floating-header { -webkit-app-region: drag; }
// .floating-header button { -webkit-app-region: no-drag; }

// 或通过 Tauri API 手动触发
import { getCurrentWindow } from '@tauri-apps/api/window';
const appWindow = getCurrentWindow();

<header onMouseDown={() => appWindow.startDragging()}>
  {/* ... */}
</header>
```

### 10.7 图标方案

不引入图标库，使用内联 SVG 组件：

```tsx
// src/components/common/Icons.tsx
export const PlayIcon = () => (
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <polygon points="5 3 19 12 5 21" fill="currentColor" />
  </svg>
);

export const PauseIcon = () => (
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <rect x="6" y="4" width="4" height="16" fill="currentColor" />
    <rect x="14" y="4" width="4" height="16" fill="currentColor" />
  </svg>
);

export const CopyIcon = () => (
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <rect x="9" y="9" width="13" height="13" rx="2" />
    <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
  </svg>
);

// 使用时颜色继承 currentColor
<button className="btn-icon">
  <PlayIcon />
</button>
```

推荐图标集参考：Lucide Icons（MIT 协议，可直接复制 SVG path）。

---

## 十一、设计检查清单

实现完成后，按以下清单验证：

- [ ] 所有颜色使用 CSS 变量，无硬编码 hex
- [ ] 玻璃面板有 backdrop-filter + 半透明背景 + 微光边框
- [ ] 顶部栏标记 `-webkit-app-region: drag`，按钮标记 `no-drag`
- [ ] 按钮有 hover / active / disabled 四态
- [ ] 输入框 focus 时有青色边框 + glow shadow
- [ ] 状态点有脉冲动画（录制/处理中）
- [ ] 答案流式输出有光标闪烁动画
- [ ] 滚动条使用自定义样式（4px 宽，半透明）
- [ ] 界面无"面试"、"AI"等敏感词汇
- [ ] 所有图标使用内联 SVG，无外部依赖
- [ ] 悬浮窗支持 resize + 拖拽
- [ ] 透明度调节通过 Tauri setOpacity
- [ ] 设置项有 label + description + 控件三层结构
- [ ] 引导页有步骤指示器
- [ ] 所有过渡动画使用 var(--ease-out) 缓动函数
