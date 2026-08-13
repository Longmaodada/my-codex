# Third-Party Notices

My Codex 本身采用 MIT License。应用还依赖多个开源软件包；各依赖仍受其自己的许可证约束。本文件列出直接依赖和研究参考，精确版本以 `package-lock.json` 与 `src-tauri/Cargo.lock`（生成后）为准，传递依赖以对应 lockfile 和随包许可证文件为准。

## 前端与构建依赖

| 项目 | 当前直接版本 | 许可证 | 来源 |
| --- | --- | --- | --- |
| React / React DOM | 19.2.0 | MIT | https://github.com/facebook/react |
| Motion for React (`framer-motion`) | 12.23.24 | MIT | https://github.com/motiondivision/motion |
| Zustand | 5.0.8 | MIT | https://github.com/pmndrs/zustand |
| Recharts | 3.2.1 | MIT | https://github.com/recharts/recharts |
| Phosphor Icons React | 2.1.10 | MIT | https://github.com/phosphor-icons/react |
| Tauri JavaScript API / CLI | 2.8.x | MIT OR Apache-2.0 | https://github.com/tauri-apps/tauri |
| Tauri plugins (autostart, global-shortcut, notification, opener, os) | 2.x | MIT OR Apache-2.0 | https://github.com/tauri-apps/plugins-workspace |
| Vite | 6.4.2 | MIT | https://github.com/vitejs/vite |
| TypeScript | 5.9.2 | Apache-2.0 | https://github.com/microsoft/TypeScript |
| Vitest | 3.2.4 | MIT | https://github.com/vitest-dev/vitest |
| Testing Library React | 16.3.0 | MIT | https://github.com/testing-library/react-testing-library |
| jsdom | 26.1.0 | MIT | https://github.com/jsdom/jsdom |

## Rust 直接依赖

| 项目 | 许可证 | 来源 |
| --- | --- | --- |
| Tauri / tauri-build | MIT OR Apache-2.0 | https://github.com/tauri-apps/tauri |
| async-trait | MIT OR Apache-2.0 | https://github.com/dtolnay/async-trait |
| chrono | MIT OR Apache-2.0 | https://github.com/chronotope/chrono |
| dirs | MIT OR Apache-2.0 | https://github.com/dirs-dev/dirs-rs |
| rusqlite | MIT | https://github.com/rusqlite/rusqlite |
| serde / serde_json | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| sha2 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| thiserror | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| tokio | MIT | https://github.com/tokio-rs/tokio |
| uuid | MIT OR Apache-2.0 | https://github.com/uuid-rs/uuid |
| walkdir | Unlicense OR MIT | https://github.com/BurntSushi/walkdir |
| pretty_assertions | MIT OR Apache-2.0 | https://github.com/rust-pretty-assertions/rust-pretty-assertions |
| tempfile | MIT OR Apache-2.0 | https://github.com/Stebalien/tempfile |

对应的 MIT 和 Apache-2.0 正文可从各项目仓库及其发布包中的许可证文件获取：

- MIT License: https://opensource.org/license/mit
- Apache License 2.0: https://www.apache.org/licenses/LICENSE-2.0
- The Unlicense: https://unlicense.org/

发布二进制时，构建流程应根据 lockfile 收集所有传递依赖的实际许可证文本；本表不能替代依赖包内的完整许可证声明。

## 研究参考

### quota-float

- 项目：https://github.com/change-42-yhmm/quota-float
- 许可证：MIT
- 使用方式：研究 Codex quota 读取、Tauri 悬浮窗口、状态降级和隐私边界。

My Codex 未将 quota-float 的 UI、品牌资产、皮肤或许可系统作为自己的内容。若后续提交直接复制或修改其代码，必须在分发物中保留相应版权和 MIT 许可声明，并在此节登记具体文件。

### OpenAI Codex

- 项目：https://github.com/openai/codex
- 许可证：Apache-2.0（以其仓库当前声明为准）
- 使用方式：参考公开 app-server 协议文档和生成类型，接入本地 Codex 进程。

My Codex 是独立项目，与 OpenAI 无官方隶属、授权、赞助或背书关系。OpenAI、ChatGPT、Codex 及相关标识属于其各自权利人。

## 用户提供的参考图

`references/widget.png` 和 `references/dashboard.png` 由项目委托方提供，仅用于本项目的视觉开发参考。它们不授权复用其中可能出现的第三方商标、Logo 或受保护产品素材；公开发布前应使用 My Codex 自有界面截图替代营销材料中的参考图。

