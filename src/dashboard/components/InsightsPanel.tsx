import { ChartDonut, ClockCounterClockwise, Database, Lightning, Robot, SquaresFour } from "@phosphor-icons/react";
import { GlassCard } from "../../components/common/GlassCard";
import type { AppSnapshot } from "../../types/analytics";
import { formatTokens } from "../../utils/format";

export function InsightsPanel({ snapshot }: { snapshot: AppSnapshot }) {
  const { usage } = snapshot;
  const bestProject = usage.projects.length
    ? usage.projects.reduce((best, item) => item.cached > best.cached ? item : best)
    : null;
  return (
    <div className="insights-column">
      <GlassCard className="mini-stats-card">
        <div className="panel-header compact"><div><span className="panel-kicker"><ChartDonut weight="fill" />今日概况</span><h2>可复用上下文</h2></div></div>
        <div className="mini-stats-grid">
          <span><SquaresFour weight="fill" /><small>Session</small><strong>{usage.sessionsToday}</strong></span>
          <span><Lightning weight="fill" /><small>Task</small><strong>{usage.tasksToday}</strong></span>
          <span><Robot weight="fill" /><small>Request</small><strong>{usage.requestsToday}</strong></span>
          <span><Database weight="fill" /><small>缓存命中</small><strong>{usage.cacheHitRatio}%</strong></span>
        </div>
      </GlassCard>

      <GlassCard className="activity-card">
        <div className="panel-header compact"><div><span className="panel-kicker"><ClockCounterClockwise weight="fill" />本地事件</span><h2>最近活动</h2></div><small>刚刚</small></div>
        <div className="activity-list">
          {usage.projects.slice(0, 4).map((project, index) => (
            <div className="activity-row" key={project.id}>
              <span className={`activity-icon activity-${index + 1}`}><SquaresFour weight="fill" /></span>
              <span><strong>{project.name}</strong><small>{project.sessions} 次会话 · {project.lastActiveAt}</small></span>
              <em>{formatTokens(project.total)}</em>
            </div>
          ))}
        </div>
      </GlassCard>

      <GlassCard className="cache-callout">
        <span className="cache-ring-mini"><strong>{usage.cacheHitRatio}%</strong><small>命中</small></span>
        <span><small>缓存表现最佳项目</small><strong>{bestProject?.name ?? "暂无数据"}</strong><em>本地统计 · Cached Input 不重复计入总 Token</em></span>
      </GlassCard>
    </div>
  );
}
