import { useState } from "react";
import { UsageTrend } from "../../components/charts/UsageTrend";
import { GlassCard } from "../../components/common/GlassCard";
import type { AppSnapshot, ProjectUsage, UsageRange } from "../../types/analytics";
import { InsightsPanel } from "../components/InsightsPanel";
import { ProjectRanking } from "../components/ProjectRanking";

const ranges = ["24H", "7D", "30D", "90D", "ALL"];
type Range = (typeof ranges)[number];

export function OverviewPage({ snapshot, onProject, defaultRange = null }: { snapshot: AppSnapshot; onProject: (project: ProjectUsage) => void; defaultRange?: "7D" | "30D" | null }) {
  const [range, setRange] = useState<Range>(defaultRange ?? "7D");
  const count = range === "24H" ? 2 : range === "7D" ? 7 : range === "30D" ? 30 : range === "90D" ? 90 : snapshot.usage.daily.length;
  const rankingRange: UsageRange = defaultRange === "30D" ? "thirtyDays" : defaultRange === "7D" ? "sevenDays" : "today";
  return (
    <div className="overview-stack">
      {defaultRange && <GlassCard className="trend-card">
        <div className="panel-header">
          <div><span className="panel-kicker">Token 使用趋势 · 本地统计</span><h2>{range === "24H" ? "今日活跃趋势" : `最近 ${range === "ALL" ? "全部" : range} 用量`}</h2></div>
          <div className="segmented compact-segmented">{ranges.map((item) => <button key={item} className={range === item ? "active" : ""} onClick={() => setRange(item)}>{item}</button>)}</div>
        </div>
        <UsageTrend data={snapshot.usage.daily.slice(-count)} />
      </GlassCard>}
      <div className="overview-columns">
        <ProjectRanking projects={snapshot.usage.projects} onSelect={onProject} initialRange={rankingRange} />
        <InsightsPanel snapshot={snapshot} />
      </div>
    </div>
  );
}
