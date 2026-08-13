import { Info } from "@phosphor-icons/react";
import type { MetricSource } from "../../types/analytics";

const labels: Record<MetricSource, string> = {
  official: "官方额度",
  local: "本地统计",
  estimated: "本地估算",
  mock: "Mock 展示",
};

const descriptions: Record<MetricSource, string> = {
  official: "来自 Codex 本地 app-server 返回的账户额度数据。",
  local: "从本机 Codex 会话事件聚合，不上传 Prompt、聊天或项目文件。",
  estimated: "基于本机会话事件关联推算，并非官方账单数据。",
  mock: "仅用于展示完整界面，不代表真实账户数据。",
};

export function SourceBadge({ source, compact = false }: { source: MetricSource; compact?: boolean }) {
  return (
    <span className={`source-badge source-${source}`} title={descriptions[source]}>
      {!compact && <Info weight="fill" aria-hidden="true" />}
      {labels[source]}
    </span>
  );
}
