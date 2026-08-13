# Phase 1：参考项目与 Codex app-server 研究

研究日期：2026-08-12

本文记录 My Codex 开发前的设计研究。结论以公开源码和官方协议文档为依据；它不是对未公开服务端接口的稳定性承诺。

## 研究资料

- [change-42-yhmm/quota-float](https://github.com/change-42-yhmm/quota-float)
- [quota-float 的 Rust Codex 适配器](https://github.com/change-42-yhmm/quota-float/blob/main/src-tauri/src/codex.rs)
- [quota-float 隐私说明](https://github.com/change-42-yhmm/quota-float/blob/main/PRIVACY.md)
- [OpenAI Codex app-server README](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md)
- [OpenAI Codex app-server v2 协议定义](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/v2.rs)
- [GetAccountRateLimitsResponse 生成类型](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/schema/typescript/v2/GetAccountRateLimitsResponse.ts)
- [RateLimitSnapshot 生成类型](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/schema/typescript/v2/RateLimitSnapshot.ts)
- [Tauri 2 窗口配置](https://v2.tauri.app/reference/config/#windowconfig)
- [Tauri 2 系统托盘](https://v2.tauri.app/learn/system-tray/)

## 1. 可复用的设计思想

### 额度状态而不是“永远有数字”

`quota-float` 对加载中、未登录、陈旧、不可用和成功做了区分，并在响应缺少可识别额度窗口时返回不可用状态。My Codex 复用这一状态机思想：解析失败不能保留成一个看似实时的新数字，也不能自动替换成 Mock。

### 多额度窗口与重置时间

Codex app-server 的 `account/rateLimits/read` 返回一个向后兼容的 `rateLimits` 视图；新版本还能返回按 `limit_id` 分桶的 `rateLimitsByLimitId`。每个窗口的关键字段是 `usedPercent`、`windowDurationMins` 和 Unix 秒级 `resetsAt`。

My Codex 内部统一为：

```text
QuotaWindow {
  id, label,
  usedPercent?, remainingPercent?,
  windowMinutes?, resetAt?
}
```

`remainingPercent = clamp(100 - usedPercent, 0, 100)`，但只在官方返回 `usedPercent` 时计算，来源仍为官方字段的派生显示。重置倒计时只根据 `resetAt` 在本地更新，不逐秒调用 API。

### 本地桌面窗口模式

参考项目验证了 Tauri 适合实现透明、无边框、置顶、跳过任务栏的轻量窗口，以及托盘菜单和悬浮球/展开态。My Codex 使用两个独立窗口标签：

- `floating`：小尺寸、无边框、置顶、跳过任务栏。
- `dashboard`：可缩放的主分析面板，默认隐藏，由悬浮窗或托盘打开。

窗口位置、Compact Mode 和用户偏好应持久化；macOS 的透明/模糊效果与 Windows Acrylic/Mica 分开做平台能力降级。

### 安全失败和响应上限

参考项目限制认证文件和 HTTP 响应大小、对敏感请求头设置敏感标记，并将认证失败、限流、网络失败和结构变化映射为安全状态。这些防御原则可以复用到任何兼容适配器中。

## 2. 不能直接复用的部分

### 不把直接读取认证文件作为默认实现

`quota-float` 当前公开实现会定位本机 `auth.json`，读取 Access Token/账户 ID，并直接请求 `chatgpt.com/backend-api/wham/usage` 等端点。这说明方案在技术上可行，但存在重要边界：

- 本地认证文件结构和私有 HTTP 端点可能随 Codex 更新变化。
- 应用进程会接触 Access Token，扩大凭据暴露面。
- 私有端点并非面向第三方应用承诺的稳定公共 API。
- 多账户、组织切换和 Token 刷新更容易出现边缘问题。

因此 My Codex 默认通过官方开源 Codex 提供的本地 app-server 协议交互，让 Codex 自己管理凭据和刷新。直接认证文件兼容层只有在未来明确启用、单独审计并告知风险时才可加入；当前设计不需要它。

### 不复制 UI、品牌资产或产品文案

用户提供的两张图片是 My Codex 的主要视觉参考。`quota-float` 只用于研究窗口、状态和数据适配方式，不机械复制其 UI、图标、皮肤、更新器或商业授权功能。

### 不把额度推算成 Token 账单

额度窗口的百分比与 Token 计数不是同一度量。不能用 `usedPercent` 反推“今日 2.9B Token”，也不能用本地 Token 总数伪造官方额度。两者在数据库和 UI 中保持独立来源。

### 不依赖实验性传输作为生产基础

官方文档将 WebSocket listener 标为实验性/不受支持。My Codex 默认使用 app-server 的本地 stdio JSONL 传输；任何实验方法必须经过初始化能力探测并允许安全降级。

## 3. My Codex 架构设计

```text
React UI
  ├─ Floating / Dashboard / Settings
  ├─ Zustand stores
  └─ source + status presentation
          │ Tauri commands/events
Rust application core
  ├─ Provider router
  │    ├─ AppServerProvider (preferred)
  │    ├─ UnavailableProvider (safe failure)
  │    └─ MockProvider (explicit only)
  ├─ Local session analytics
  ├─ Refresh coordinator (single-flight/backoff)
  ├─ SQLite repository
  └─ Tray / Window / Settings / Notifications
          │
  Local Codex app-server + local session/event files
```

核心契约：

- `QuotaProvider` 只负责账户和额度快照，不承担本地项目排行。
- `SessionAnalytics` 只提取允许的结构化元数据，立即丢弃文本正文。
- `SnapshotService` 将官方、本地、估算和 Mock 组合成带来源的 `AppSnapshot`。
- `RefreshCoordinator` 保证同一时刻只有一次官方刷新，失败退避为 10/30/60/120/300 秒。
- UI 不从数值猜来源；每个指标都携带 `source` 和 `status`。

## 4. 数据采集方案

### 官方数据路径

1. 启动 `codex app-server --stdio`，完成 `initialize` / `initialized` 握手。
2. 调用 `account/read`，不强制刷新时使用 `refreshToken: false`。
3. 调用 `account/rateLimits/read` 获取额度快照。
4. 优先选择 `rateLimitsByLimitId.codex`；没有多桶数据时使用向后兼容的 `rateLimits`。
5. 订阅 `account/rateLimits/updated`。官方说明该通知是稀疏滚动更新，所以只合并实际出现的字段；不能用 `null` 随意清空最近快照。
6. 若能力探测支持，调用 `account/usage/read` 取得账户 Token 活动摘要和每日桶。
7. 对超载错误采用带抖动的指数退避；保留 `lastSuccessAt` 并显示 stale。

### 本地分析路径

- 扫描明确允许的 Codex session/event 位置，增量读取、按文件游标避免反复全量解析。
- 仅抽取时间、Token 计数、模型、工作目录/项目标识、结构化 Skill 调用标识和缓存字段。
- 不保存 Prompt、assistant 回复、reasoning 正文、工具参数中的文件内容或原始事件。
- 项目/Skill/模型/缓存聚合写入本地 SQLite；关联不可靠时标记 `estimated` 和置信度。
- 统计与官方额度保持两个命名空间，避免语义混淆。

### Mock 路径

- 浏览器预览自动 Mock；桌面端只能由明确设置启用。
- Mock 快照带 `provider=mock`、`source=mock`，UI 显示固定标记。
- 离开 Mock 后如果真实适配器失败，显示不可用，不继续显示旧 Mock。

完整矩阵见 [data-sources.md](data-sources.md)。

## 5. 文件目录

```text
my-codex/
├─ references/                 视觉参考图
├─ docs/                       研究、来源、隐私、验收
├─ scripts/                    Windows 构建与站点辅助脚本
├─ src/
│  ├─ components/              通用卡片、来源徽标、Tooltip
│  ├─ dashboard/ floating/     两个主要窗口
│  ├─ charts/ projects/ skills/ cache/ settings/
│  ├─ hooks/ stores/ services/ types/ utils/
│  └─ data/                    明确标识的 Mock 数据
└─ src-tauri/
   ├─ capabilities/
   └─ src/
      ├─ quota/ codex/ session/ analytics/
      ├─ database/ settings/ commands/
      └─ tray/ window/ platform/ notifications/
```

目录可按 Rust 模块规模折叠，但职责边界不能退化为把全部逻辑塞入 `main.rs`。

## 6. 开发阶段计划

1. 研究与协议边界：本文和数据/隐私契约。
2. Tauri 初始化：双窗口、权限、命令桥接。
3. Quota Adapter：app-server 能力探测、安全失败、Mock Provider。
4. SQLite：迁移、聚合表、隐私字段白名单。
5. 悬浮窗：额度、重置倒计时、90 天热力图、Compact Mode。
6. Dashboard：核心卡片、趋势和异常状态。
7. Project Analytics：本地项目识别、排行与详情。
8. Skill Analytics：结构化关联、置信度和“本地统计”标签。
9. Cache/Model Analytics：缓存、模型、请求/会话聚合。
10. 动画与 UI Polish：数字缓动、卡片 stagger、背景光和深色模式。
11. 托盘和设置：自启、快捷键、通知、窗口偏好。
12. 自动化与人工测试：协议、数据库、UI、隐私、性能。
13. Windows Installer：在完整 MSVC/SDK 环境生成并验收 EXE、NSIS 和 MSI。

每个阶段的完成标准不是“文件存在”，而是相应自动测试通过且人工场景有证据。当前状态以 [acceptance.md](acceptance.md) 为准。

