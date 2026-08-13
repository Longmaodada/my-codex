# 数据来源与准确性边界

My Codex 的核心规则是：任何数字都必须知道自己从哪里来。数据对象使用 `source`，采集过程使用 `status`；UI 必须同时尊重两者。

## 来源枚举

| `source` | UI 标签 | 定义 |
| --- | --- | --- |
| `official` | 官方数据 | 由本机 Codex app-server 从当前账户返回，或只对该官方字段做无歧义格式转换 |
| `local` | 本地统计 | 从本机 Codex session/event 的结构化元数据聚合，不代表官方账单 |
| `estimated` | 估算 | 使用公开算法从本地事件推断，必须显示置信度或算法说明 |
| `mock` | Mock | 固定演示/测试数据，必须显著标识，禁止与真实快照混合显示 |

## 状态枚举

| `status` | 展示规则 |
| --- | --- |
| `loading` | 骨架或加载动画，不先显示 Mock 数字 |
| `ok` | 显示数值、来源和采样时间 |
| `stale` | 可显示最后成功值，但必须显示“数据已过期”和 `lastSuccessAt` |
| `signed_out` | 数值显示 `—`，提示先在 Codex 登录 |
| `unavailable` | 数值显示 `—`，给出网络/进程/能力不可用原因 |
| `schema_changed` | 数值显示 `—`，明确协议结构可能变化，禁止猜字段 |

## 指标矩阵

| 指标 | 首选来源 | 降级 | 说明 |
| --- | --- | --- | --- |
| 当前套餐 | official | 不可用 | `account/read` / `account/updated` 的 `planType`；不能从 UI 文案猜测 |
| 主/次额度已用百分比 | official | stale → 不可用 | `account/rateLimits/read` 返回的窗口 |
| 剩余额度百分比 | official 派生 | stale → 不可用 | 仅计算 `100 - usedPercent` 并钳制到 0–100 |
| 额度重置时间 | official | stale → 不可用 | `resetsAt` 为 Unix 秒；倒计时本地计算 |
| 今日/每日 Token | official（可用时）或 local | 不可用 | 官方 `account/usage/read` 与本地统计不可悄悄相加；UI 显示来源 |
| Input/Output/Cached/Reasoning | local 或 official | 缺失字段为 `—` | 只使用实际存在的分类字段，`total` 不能重复计算缓存 Token |
| 项目排行 | local | 不可用 | 按 session 工作目录/项目标识聚合，不是官方账单 |
| Skill 排行 | local | estimated | 优先结构化 Skill 事件；名称匹配等启发式必须为 estimated |
| 模型排行 | local | 不可用 | 使用 session/event 中明确模型标识 |
| 缓存命中率 | local | 不可用 | 分母和缓存字段缺失时不显示 0% |
| 请求/Session/Task | local | 不可用 | 使用明确事件边界去重 |
| 活跃时长 | estimated | 不可用 | 根据事件间隔与 idle cutoff 计算，不等于实际工作时长 |
| 90 天热力图 | local/official | 空态 | 每日桶保持单一来源；Tooltip 显示来源 |

## Official Provider

默认路径是本机 `codex app-server --stdio` 的 JSONL RPC：

- `account/read`：账户类型、套餐和是否需要 OpenAI 认证。
- `account/rateLimits/read`：额度窗口初始快照。
- `account/rateLimits/updated`：额度稀疏更新。
- `account/usage/read`：当前版本支持时的账户 Token 活动摘要和每日桶。

Provider 必须先完成 `initialize`，再发送 `initialized`。协议 schema 与运行中的 Codex 版本绑定，适配器应做能力探测，不假设所有版本拥有相同字段。

多桶额度存在时优先 `rateLimitsByLimitId.codex`；否则只在向后兼容快照确实代表 Codex 限额时使用 `rateLimits`。任何实验字段均需显式 capability opt-in；拒绝或不存在时安全降级。

## Local Session Analytics

### 允许抽取

- 事件/会话 ID 的本地不可逆标识。
- 时间戳和明确的 turn/session/task 边界。
- Input、Output、Cached Input、Reasoning、Total Token 数。
- 模型名称、工作目录/项目标识。
- 结构化 Skill 名称、调用结果和关联项目。
- 请求数、缓存字段、错误分类等非正文元数据。

### 必须丢弃

- 用户 Prompt、assistant 回复、reasoning 正文。
- shell 命令正文、补丁正文、工具参数内的文件内容。
- 项目源代码、附件、图片/音频内容、原始会话 JSONL。
- Access Token、Refresh Token、Cookie、账户 ID 和认证文件路径。

解析器采用字段白名单，而不是“读取全部后再过滤”。聚合完成后只保存聚合结果与必要游标；错误日志不能包含原始行。

## 去重与计算

- Token 聚合键至少包含稳定事件 ID；同一 session 文件被重复扫描不能重复计数。
- 文件增量游标记录本地文件标识、偏移/安全指纹和最后处理时间；文件截断或轮转时重新校验。
- `total` 优先使用来源中明确的总计；否则按协议语义计算。Cached Input 通常属于 Input 的组成部分，不能无条件再次相加。
- 缓存命中率只在分母大于 0 且字段语义明确时计算。
- 活跃时长使用相邻事件间隔并设置 idle cutoff；该值始终为 `estimated`。
- Skill 只有结构化调用证据时为 `local`；字符串匹配或上下文推断为 `estimated`，并保存 0–1 置信度。
- 日界线和 7/30/90 天窗口按用户本地时区计算，数据库时间戳保存为 UTC。

## 刷新与陈旧规则

- 手动刷新始终可用，但服从 single-flight；已有请求运行时复用结果或跳过新请求。
- 智能模式建议：活跃 10 秒、普通 30 秒、全部窗口隐藏 60 秒、reset 前 5 分钟 10 秒。
- 失败退避：10 秒、30 秒、60 秒、120 秒，最大 5 分钟，并加入小幅 jitter。
- 收到稀疏 `account/rateLimits/updated` 时，只更新出现的字段；未知或缺失字段不能覆盖已知字段。
- 最后成功快照超过产品定义的 TTL 后变为 stale；新鲜度和额度 reset 倒计时是两个概念。

## Mock 隔离

Mock 数据仅用于浏览器预览、视觉回归和显式演示模式：

- 快照根节点、卡片和 Tooltip 均能识别 `source=mock`。
- Mock 数据库与真实本地数据库不得共用统计记录。
- 关闭 Mock 后清空内存 Mock 快照并立即请求真实 Provider。
- 真实 Provider 失败时显示不可用，绝不以 Mock 填洞。

## 兼容性策略

Codex app-server 协议可能随版本扩展。适配器应保存最小必要兼容信息（Codex 版本、schema 能力、采样时间），而不是保存原始响应。未知字段忽略，已知必需字段类型错误则返回 `schema_changed`。兼容性测试需要覆盖至少一个当前版本、一个缺少 `account/usage/read` 的版本以及协议错误样例。

