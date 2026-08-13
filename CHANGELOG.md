# Changelog

## [0.1.2] - 2026-08-13

### Fixed

- 胶囊支持悬停后自动展开，鼠标离开 My Codex 悬浮窗后自动收起。
- 保留胶囊单击与双击展开入口，并修复拖动时误触发展开的问题。
- 胶囊和展开后的 My Codex 窗口都可以继续拖动。

### Distribution

- 本版本目标是 Windows 绿色版：直接运行 `my-codex.exe`，不再制作安装包。
- 绿色版仍需要 Windows WebView2 运行时；首次运行前请解压完整目录。

## [0.1.1] - 2026-08-13

### Fixed

- 修复点击胶囊展开 My Codex 后因误触发自动收起而立即闪回的问题。
- 生成并验证 Windows 版本安装包。

本项目的显著变更记录在此文件。格式参考 Keep a Changelog，版本号遵循 Semantic Versioning。

## [Unreleased]

### Added

- My Codex 双窗口桌面应用架构：额度悬浮窗与 Usage Intelligence Dashboard。
- Codex app-server 优先的 Provider 设计、本地 Session Analytics 与显式 Mock Provider。
- 官方、本地、估算和 Mock 四类数据来源，以及 loading/stale/signed-out/unavailable/schema-changed 状态。
- 项目、Skill、模型、缓存、90 天热力图和设置所需的数据结构。
- Windows NSIS/MSI 打包配置与 `scripts/build-windows.ps1` 可重复构建门禁。
- 用户提供的悬浮窗和 Dashboard 视觉参考图。
- 参考研究、数据来源、隐私边界和验收文档。

### Security

- 默认认证路径由 Codex app-server 管理，My Codex 不要求重新输入 OpenAI 密码。
- 本地分析采用元数据白名单；Prompt、回复、项目文件内容和 Token 不进入统计数据库。
- Provider 失败时显示不可用或陈旧状态，不用未标识的 Mock 数据填充。

### Known limitations

- 本次执行环境没有 Rust、MSVC 和 Windows SDK，尚未产出或验证 Windows 主程序、NSIS 安装包或 MSI。
- 真实登录账户的在线 quota 端到端验收尚未完成。
- 本地项目/Skill/模型/缓存统计不等同于 OpenAI 官方账单。
