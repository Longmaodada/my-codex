# My Codex

My Codex 是一个本地优先的 Codex 配套桌面应用，提供额度悬浮胶囊、Token 使用统计、项目/Skill/模型分析和缓存分析。桌面端使用 Tauri 2、Rust、React 和 TypeScript。

## 当前版本：0.1.3

### 悬浮胶囊交互

- 鼠标进入胶囊时展开 My Codex，并播放弹出动画。
- 鼠标停留在 My Codex 内时保持展开。
- 鼠标离开 My Codex 后立即播放缩小收回动画，回到胶囊。
- 展开和收起只由悬停控制；点击、双击和定时器不会改变界面状态。
- “锁定位置”只控制是否允许移动，不参与展开或收起，也不会改变窗口尺寸。
- 胶囊中的百分比和额度圆环作为整体居中，圆环在当前阈值内保持单色。

### 数据与隐私

- 真实模式通过本机 Codex app-server 获取可用额度。
- 未登录或接口不可用时显示明确的不可用状态，不伪造真实数据。
- Mock 数据仅用于浏览器预览和界面测试，并带有 Mock 标识。
- 应用不要求用户再次输入 OpenAI 密码，也不持久化 Access Token。
- 本地统计只保留允许的结构化元数据，不保存 Prompt、回复或项目文件内容。

## 开发

安装依赖：

```powershell
npm.cmd install
```

启动浏览器预览：

```powershell
npm.cmd run dev
```

启动 Tauri 桌面开发版：

```powershell
npm.cmd run tauri:dev
```

## 验证

```powershell
npm.cmd run typecheck
npm.cmd test
npm.cmd run build
npm.cmd run test:sites
```

Windows release 构建：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-windows.ps1
```

构建脚本会执行 TypeScript 检查、前端测试、Rust 检查、Rust 测试以及 Tauri NSIS/MSI 打包。需要 Node.js 20+、Rust MSVC 工具链、Visual Studio 2022 C++ 构建工具、Windows SDK 和 WebView2 Runtime。

当前已验证的构建产物：

```text
src-tauri/target/release/my-codex.exe
src-tauri/target/release/bundle/nsis/My Codex_0.1.3_x64-setup.exe
src-tauri/target/release/bundle/msi/My Codex_0.1.3_x64_en-US.msi
```

发布时请保留 Tauri 运行所需的完整目录或使用安装包，不要只复制主 exe。

## 目录

```text
src/                    React UI、stores、hooks 和 services
src-tauri/src/          Rust 数据适配、数据库、托盘和窗口控制
tests/                  Sites 与前端相关测试
docs/                   研究、数据来源、隐私和验收文档
scripts/                Windows 构建与 Sites 准备脚本
```

## 变更说明

详细变更见 [CHANGELOG.md](CHANGELOG.md)。本版本重点修复悬停状态与锁定状态混用的问题：窗口设置保存不再无条件重设胶囊尺寸，只有真正切换胶囊模式时才调整窗口大小。
