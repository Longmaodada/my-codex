# Changelog

## [1.0.0] - 2026-08-22

### Added

- 新增 Codex 官方额度重置卷信息展示，并明确区分官方额度信息与 My Codex 本地统计、设置及软件数据。
- 官方重置卷信息不可用时提供明确的刷新提示，不将“暂不可用”误报为本地数据重置。

### Changed

- 统一 npm、Tauri、Rust crate 与 VERSION 的正式版本号为 1.0.0。
- Windows 发布脚本增加版本链校验，并在构建完成后验收可交付的 EXE 安装包。

## [0.1.5] - 2026-08-22

### Added

- 展示 Codex 官方发放的额度重置卷数量及每张重置卷的到期日期。

## [0.1.4] - 2026-08-14

### Fixed

- Keep the expanded My Codex surface visible after the floating window position is locked; it no longer returns to the capsule on pointer leave.

## [0.1.3] - 2026-08-13

### Fixed

- 悬浮胶囊只在鼠标进入时展开，带弹出动画；鼠标离开 My Codex 后立即收回胶囊。
- 移除点击、双击、定时器等非悬停展开路径。
- 修复锁定位置触发窗口缩回的问题；锁定现在只控制是否允许移动。
- 配额圆环改为当前阈值对应的整体单色，不再同时显示多段颜色。
- 启动优先显示本地缓存，后台刷新额度，减少首次启动卡顿。
- 修复胶囊内容左右不对称，百分比与圆环整体居中。
- 修复 Windows 构建脚本在 PowerShell 下的 `vswhere` 路径解析问题。

### Verification

- TypeScript 检查通过。
- 前端测试 8/8 通过。
- Rust 测试 10/10 通过。
- Sites 测试 4/4 通过。
- Windows release、NSIS 和 MSI 构建通过。

## [0.1.2] - 2026-08-13

### Fixed

- 胶囊支持悬停展开，鼠标离开 My Codex 后自动收起。
- 修复拖动时误触发展开的问题。

## [0.1.1] - 2026-08-13

### Fixed

- 修复悬浮窗展开后因误触发自动收起而立即闪回的问题。

本项目的显著变更记录在此文件。格式参考 Keep a Changelog，版本号遵循 Semantic Versioning。
