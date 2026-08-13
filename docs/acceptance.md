# 验收清单

本文是 My Codex 的完成定义。状态值：`通过`、`失败`、`待验收`、`不适用`。只有带可复现命令或人工证据的项目才能标记为通过。

## 当前事实快照（2026-08-12）

| 项目 | 状态 | 证据/说明 |
| --- | --- | --- |
| 研究、数据来源、隐私文档 | 通过 | `docs/reference-research.md`、`docs/data-sources.md`、`docs/privacy-boundary.md` |
| Windows 构建脚本语法与前置检查 | 通过 | PowerShell 语法解析通过；实际执行在缺少 `rustc.exe` 时明确失败且未伪造产物 |
| 浏览器 Mock UI | 通过 | `npm.cmd run typecheck`、`npm.cmd test`、`npm.cmd run build`、`npm.cmd run test:sites` 全部通过 |
| Tauri 桌面启动 | 待验收 | 当前环境无 Rust/MSVC/Windows SDK |
| 真实 Codex 登录与在线额度 | 待验收 | 未使用真实登录账户做端到端验证 |
| Windows 主程序/安装器 | 失败 | 当前环境未生成 `My Codex.exe`、NSIS `.exe` 或 `.msi`；不得称为已交付安装包 |

## 自动化门禁

在仓库根目录逐项执行并记录退出码：

| 检查 | 命令 | 当前状态 |
| --- | --- | --- |
| TypeScript | `npm.cmd run typecheck` | 通过 |
| 前端单元测试 | `npm.cmd run test` | 通过（3 文件、7 项） |
| Web/Sites 构建 | `npm.cmd run build` | 通过 |
| Sites Worker | `npm.cmd run test:sites` | 通过（4 项） |
| Rust 编译检查 | `cargo check --manifest-path .\src-tauri\Cargo.toml` | 待验收（当前无 cargo） |
| Rust 测试 | `cargo test --manifest-path .\src-tauri\Cargo.toml` | 待验收（当前无 cargo） |
| Tauri 打包 | `npm.cmd run tauri:build` | 待验收（当前无原生工具链） |

一键 Windows 门禁：

```powershell
.\scripts\build-windows.ps1
```

## 数据正确性

- [ ] `account/read` 的未登录结果映射为 `signed_out`，所有真实数值为 `—`。
- [ ] `account/rateLimits/read` 的 primary/secondary 和多桶 `codex` 数据映射正确。
- [ ] `usedPercent` 到剩余百分比的转换覆盖 0、边界、100 和异常值。
- [ ] `resetsAt` 以 Unix 秒解析；时区显示正确；倒计时每秒更新但不触发 API 请求。
- [ ] 稀疏 `account/rateLimits/updated` 不会用缺失/未知字段清空现有快照。
- [ ] 同时触发自动/手动刷新时只有一个在途请求。
- [ ] 失败退避达到 10/30/60/120/300 秒上限，并可在成功后复位。
- [ ] stale 显示最后成功时间；schema 变化显示不可用而非默认数字。
- [ ] 本地 session 重扫不重复计数。
- [ ] Cached Token 不被重复加入 Total。
- [ ] Skill 的结构化证据为 local；启发式关联为 estimated 且显示置信度。
- [ ] 真实/本地/估算/Mock 来源在卡片和 Tooltip 中一致。

## Mock Mode

- [ ] 浏览器预览显著显示 Mock。
- [ ] 桌面端默认不因真实 Provider 失败而自动启用 Mock。
- [ ] 启用/关闭 Mock 后数据和标识同步切换，不出现混合快照。
- [ ] Mock 数据库或内存状态不污染真实本地统计。

## 悬浮窗人工验收

- [ ] 尺寸/DPI 在 100%、125%、150%、200% 下无裁切。
- [ ] Frameless、透明、圆角、柔和阴影和毛玻璃按平台能力工作。
- [ ] Always On Top、跳过任务栏、拖动、锁定位置和位置记忆有效。
- [ ] 首次圆环 700–1200ms ease-out；额度变化不闪屏。
- [ ] Token 数字 Count Up，刷新图标只在真实刷新期间旋转。
- [ ] 90 天热力图为 13 周左右，每格 Tooltip 字段齐全。
- [ ] 底部只有“打开主面板 / 设置 / 退出”三个按钮。
- [ ] Compact Mode 约 110 × 48，Hover spring 展开且可关闭自动收起。
- [ ] 三类异常：未登录、断网、结构变化均不显示假数字。

## Dashboard 人工验收

- [ ] 1360 × 860 首屏及最小 1050 × 680 无关键内容遮挡。
- [ ] 顶部额度/今日/近 7 天/累计卡片层级清楚。
- [ ] Tab 局部切换，不重载整个窗口。
- [ ] 24H/7D/30D/90D/ALL 趋势 Tooltip 显示 Token 分类。
- [ ] 项目排行支持时间范围、排序、Top1/2/3 和详情。
- [ ] Skill、Cache、Model 页面对不可获得字段显示 `—` 和来源。
- [ ] Light/Dark/System 三种主题可切换；Dark 非纯黑。
- [ ] 动画在 `prefers-reduced-motion` 下减少或关闭。
- [ ] 键盘导航、焦点环、对比度和 Tooltip 可访问性检查通过。

## 托盘、设置与通知

- [ ] 托盘菜单各动作可用，Tooltip 显示真实剩余额度或不可用状态。
- [ ] 关闭 Dashboard 时按设置隐藏到托盘，不误退出后台。
- [ ] 开机启动启停后状态与操作系统一致。
- [ ] `Ctrl+Shift+Q` 与 `Ctrl+Shift+D` 冲突时给出可见错误。
- [ ] 阈值通知同一额度周期不重复轰炸，reset 后正确复位。
- [ ] 退出会终止 app-server 子进程并提交数据库事务。

## 隐私验收

- [ ] SQLite 全库搜索不含测试 Prompt、assistant 正文、Bearer/JWT 和原始事件。
- [ ] stdout/stderr 与应用日志不含 Token、账户 ID、Prompt 或完整私有路径。
- [ ] 断网抓包只出现预期的本机 IPC/app-server 通信；My Codex 无第三方遥测。
- [ ] 恶意项目名/Skill 名按纯文本显示，不产生 HTML 注入。
- [ ] 清除本地统计后聚合表、游标、缓存快照消失，设置保留策略符合提示。
- [ ] CSP 阻止远程脚本；外链只允许已校验的 `https` 地址。

## 性能验收

在 Release 构建、动画稳定 60 秒后测量：

| 目标 | 验收方式 |
| --- | --- |
| 悬浮窗 Idle CPU < 1% | Windows Performance Monitor，5 分钟平均值 |
| 内存尽可能 < 150 MB | Task Manager/Process Explorer，记录 working set |
| 冷启动 < 2 秒 | 从进程创建到悬浮窗首个可交互帧 |
| Dashboard 首屏 < 500 ms | 从打开命令到关键卡片渲染 |
| 本地查询 < 100 ms | 90 天聚合和排行查询的 p95 |
| 动画 60 FPS | DevTools Performance，正常硬件无持续长任务 |

达不到目标时记录机器规格和 p50/p95，不通过隐藏或删掉测量来“达标”。

## Windows 安装器验收

- [ ] `src-tauri/target/release/my-codex.exe` 存在且可独立启动。
- [ ] NSIS `.exe` 与 MSI `.msi` 均由当前提交生成。
- [ ] 记录 SHA-256、版本、构建时间和 Git commit。
- [ ] 普通用户权限安装、启动、升级、卸载和重装通过。
- [ ] 安装路径含空格和中文用户名场景通过。
- [ ] WebView2 缺失场景提示正确。
- [ ] 若公开发布，Authenticode 签名及时间戳验证通过；未签名时明确告知 SmartScreen 风险。
- [ ] 干净 Windows 11 虚拟机完成一次真实安装验收。

在以上安装器项目有证据前，README 和发布说明必须继续保留“未产出/未验证安装器”的声明。
