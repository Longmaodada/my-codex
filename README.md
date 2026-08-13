# My Codex

My Codex 是一款本地优先的 Codex 配套桌面应用，用于查看 Codex 额度、Token 使用情况和本地开发活动分析。它提供一个轻量的桌面悬浮胶囊，以及一个功能完整的 My Codex Dashboard，帮助用户在不离开开发环境的情况下了解额度和使用趋势。

桌面端使用 Tauri 2、Rust、React 和 TypeScript 构建。应用以 Windows 为主要交付平台，同时保留跨平台架构边界。浏览器预览使用明确标识的 Mock 数据，桌面真实模式通过本机 Codex app-server 获取可用信息。

## 功能概览

### 额度悬浮胶囊

- 显示当前主要额度窗口的剩余百分比和重置时间。
- 鼠标进入胶囊时展开 My Codex，并播放弹出动画。
- 鼠标停留在 My Codex 内时保持展开。
- 鼠标离开 My Codex 后立即播放缩小收回动画，回到胶囊界面。
- 展开和收起只由悬停控制，不使用点击、双击或自动定时器改变界面状态。
- 支持锁定悬浮窗位置；锁定只控制是否允许移动，不影响展开和收起。
- 配额圆环根据当前额度阈值显示整体单色，避免多段颜色造成误读。

### My Codex Dashboard

- 查看今日、近 7 天和累计 Token 使用情况。
- 查看输入、输出、缓存和推理 Token 分项数据。
- 查看近 90 天使用热力图和累计用量。
- 查看项目、Skill、模型和缓存分析。
- 查看额度窗口、重置时间、数据来源和刷新状态。
- 支持主题、刷新模式、通知阈值、置顶和悬浮窗显示等设置。
- 支持托盘、全局快捷键、开机启动和窗口控制。

### 数据来源与状态保护

应用区分以下数据来源：

- 官方数据：Codex app-server 返回的账户、套餐和额度信息。
- 本地统计：从本机会话事件中提取允许的结构化元数据后聚合。
- 估算数据：基于明确规则推导出的本地指标。
- Mock 数据：仅用于浏览器预览和界面测试，并带有明确标识。

当用户未登录、接口不可用、协议字段发生变化或数据过期时，应用显示明确的不可用或陈旧状态，不使用未标识的数字冒充真实数据。

## 本地优先与隐私

- My Codex 不要求用户再次输入 OpenAI 密码。
- 认证凭据由本机 Codex 管理，应用不持久化 Access Token。
- 不收集或上传 Prompt、回复正文、聊天原文、项目源码或文件内容。
- 本地分析只保留必要的结构化元数据和聚合指标。
- 不包含第三方广告、行为追踪或远程统计服务。
- 删除本地应用数据后不会从云端恢复，因为项目不建立独立的云端同步账户。

详细边界见 [隐私边界](docs/privacy-boundary.md) 和 [数据来源说明](docs/data-sources.md)。

## 工作方式

桌面程序包含两个窗口：

1. Floating Widget：显示额度和快速入口的悬浮窗口。
2. Dashboard：显示详细统计、分析和设置的主面板。

启动时优先读取本地缓存，让界面尽快显示；额度和本地活动数据在后台刷新，完成后通过事件更新界面。刷新过程采用 single-flight 机制，避免两个窗口同时重复请求。

## 安装和运行

### 浏览器预览

```powershell
npm.cmd install
npm.cmd run dev
```

浏览器预览默认使用 Mock 数据，仅用于检查界面和交互。

### Tauri 桌面开发版

```powershell
npm.cmd run tauri:dev
```

桌面开发需要 Node.js 20+、Rust stable MSVC 工具链、Visual Studio C++ 构建工具、Windows SDK 和 WebView2 Runtime。

### Windows release 构建

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-windows.ps1
```

构建脚本会依次执行：

- TypeScript 类型检查
- 前端测试
- Rust 编译检查
- Rust 测试
- Vite 生产构建
- Sites 文件准备
- Tauri release 构建
- NSIS 和 MSI 打包

构建产物位于：

```text
src-tauri/target/release/my-codex.exe
src-tauri/target/release/bundle/nsis/*.exe
src-tauri/target/release/bundle/msi/*.msi
```

发布时请使用完整的 Tauri 运行目录或安装包，不要只复制单独的主 exe。

## 验证命令

```powershell
npm.cmd run typecheck
npm.cmd test
npm.cmd run build
npm.cmd run test:sites
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml
```

## 项目结构

```text
src/                    React UI、stores、hooks、services 和 types
src-tauri/src/          Rust 适配器、数据库、额度服务、托盘和窗口控制
tests/                  Sites 和相关测试
docs/                   数据来源、隐私边界和验收文档
scripts/                Windows 构建与 Sites 准备脚本
worker/                 Sites 服务端入口
public/                 Web 公共资源
```

## 许可证与说明

My Codex 是独立的本地工具，与 OpenAI 没有官方隶属关系。项目中的官方数据能力取决于本机 Codex 版本、登录状态和可用的 app-server 协议。项目/Skill/模型/缓存统计是本地分析结果，不等同于 OpenAI 官方账单。
