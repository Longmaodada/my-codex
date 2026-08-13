import type { TokenBreakdown } from "../../types/analytics";
import { formatTokens } from "../../utils/format";

const metrics = [
  { key: "input", label: "输入", color: "blue" },
  { key: "output", label: "输出", color: "violet" },
  { key: "cached", label: "缓存", color: "amber" },
  { key: "reasoning", label: "思考", color: "rose" },
] as const;

export function MetricLegend({ data, compact = false }: { data: TokenBreakdown; compact?: boolean }) {
  return (
    <div className={`metric-legend ${compact ? "metric-legend-compact" : ""}`}>
      {metrics.map((metric) => (
        <div className="metric-legend-item" key={metric.key}>
          <span className={`metric-dot dot-${metric.color}`} />
          <span>{metric.label}</span>
          <strong>{formatTokens(data[metric.key])}</strong>
        </div>
      ))}
    </div>
  );
}
