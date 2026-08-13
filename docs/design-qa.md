# 视觉验收记录

## 证据

- 参考图：[widget.png](../references/widget.png)（200 × 349 px）、[dashboard.png](../references/dashboard.png)（441 × 394 px）。
- 实现截图：[widget-280x430.png](qa/widget-280x430.png)（280 × 430 CSS px，DPR 1）、[dashboard-1280x800.png](qa/dashboard-1280x800.png)（1280 × 800 CSS px，DPR 1）。
- 合并对照页面：[comparison.html](qa/comparison.html)。参考照片与实现截图使用 contain 对齐，按布局比例、层级、信息密度和卡片关系比较，不按压缩照片的像素颜色作结论。

## 检查结果

两张实现图均完成全视图检查。悬浮窗保留 280 × 430 主态、110 × 48 Compact、顶部额度/今日 Token/目标/重置/90 日热力图/三个底部按钮；Dashboard 保留四张摘要卡、Tab、左侧项目排行和右侧概况/活动区。重点区域检查覆盖圆环、热力格、进度条、来源徽章、排行榜行和底部按钮。

## 迭代记录

- P1：初版 Dashboard 在“今日任务”首屏显示趋势图，压缩了参考图中的项目排行与右侧洞察；已改为今日首屏直接显示双栏排行/洞察，趋势移至月度/周趋势。
- P1：初版 Tauri floating URL 未识别 `window=floating`，会落到 Dashboard；已统一识别 `widget` 与 `floating`。
- P2：初版品牌标识为代码绘制/通用图标；已替换为项目图标资源，并生成 Tauri 多平台图标。
- P2：初版热力图与底部按钮在 280 × 430 中有轻微压缩；已减少热力卡高度、固定格子尺寸并加固底部按钮背景。

最终结果：通过。仍保留 Mock 横幅和来源标签，这是数据边界要求，不是视觉缺陷。
