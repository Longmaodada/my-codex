import type { DailyUsage } from "../../types/analytics";
import { formatDuration, formatTokens } from "../../utils/format";

function level(value: number, max: number) {
  if (value === 0) return 0;
  const ratio = value / Math.max(1, max);
  if (ratio < 0.22) return 1;
  if (ratio < 0.48) return 2;
  if (ratio < 0.72) return 3;
  return 4;
}

export function Heatmap({ data, weeks = 13 }: { data: DailyUsage[]; weeks?: number }) {
  const visible = data.slice(-(weeks * 7));
  const max = Math.max(...visible.map((day) => day.total), 1);
  return (
    <div className="heatmap-wrap">
      <div className="heatmap-weekdays" aria-hidden="true">
        {["一", "二", "三", "四", "五", "六", "日"].map((day) => <span key={day}>{day}</span>)}
      </div>
      <div className="heatmap" style={{ gridTemplateColumns: `repeat(${weeks}, minmax(0, 1fr))` }} role="img" aria-label={`近 ${weeks * 7} 天 Token 使用热力图`}>
        {visible.map((day) => (
          <span
            className={`heat-cell heat-${level(day.total, max)}`}
            key={day.date}
            tabIndex={0}
            aria-label={`${day.date}，${formatTokens(day.total)} Token`}
          >
            <span className="heat-tooltip">
              <strong>{day.date.replaceAll("-", "/")}</strong>
              <span>Token：{formatTokens(day.total)}</span>
              <span>请求：{day.requests}</span>
              <span>缓存命中：{day.cacheHitRatio}%</span>
              <span>使用时长：{formatDuration(day.activeSeconds)}</span>
              <em>本地统计</em>
            </span>
          </span>
        ))}
      </div>
    </div>
  );
}
