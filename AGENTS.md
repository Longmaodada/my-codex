# Prototype Instructions

Run the local server yourself and open the preview in the browser available to this environment. Do not give the user server-start instructions when you can run it.

Before making substantial visual changes, use the Product Design plugin's `get-context` skill when the visual source is unclear or no longer matches the current goal. When the user gives durable prototype-specific design feedback, preferences, or decisions, record them in `AGENTS.md`.

When implementing from a selected generated mock, treat that image as the source of truth for layout, component anatomy, density, spacing, color, typography, visible content, and hierarchy.

Prototype feedback to preserve: the floating capsule does not show the app icon; its quota meter uses a continuous deep-blue-to-red gradient with visible high/mid/low bands; keep the existing capsule transition animation while keeping the native drag surface stable; clip transparent/backdrop overflow so neither the capsule nor the My Codex window shows corner traces.

Current interaction decision: the capsule expands to the My Codex state only while the pointer is over it, and collapses when the pointer leaves. Click, double-click, drag gestures, and other expansion triggers are intentionally disabled.

Latest prototype feedback: the capsule must always display the percentage before the hollow quota ring, with no mojibake text. Both the capsule and My Codex dashboard must remain draggable through their non-interactive surfaces.

Latest interaction feedback: after the floating window position is locked, keep the expanded My Codex surface visible and do not return to the capsule. Unlocking leaves the surface expanded until the pointer leaves it.

Usage separation: Skill usage rankings appear only on the Skill page. The 今日任务, 月度趋势, and 逐任务记录 pages show task/token/trend data without Skill usage chips or rankings.

Dashboard information architecture: 今日任务 keeps the project usage ranking and insights in the primary view; 月度趋势 is the time-series trend view; 逐任务记录 is the separate page for today's per-task ledger. Keep these surfaces distinct so the ledger does not push the project ranking below the fold.

Build app UI in `src/`. Keep `.openai/hosting.json`, `worker/index.js`, `scripts/prepare-sites-build.mjs`, and `tests/sites-worker.test.mjs` intact so the same local prototype can be handed to Sites. Before a Sites handoff, run `npm run build` and `npm run test:sites`; the build must leave `dist/client/index.html`, `dist/server/index.js`, and `dist/.openai/hosting.json`.
