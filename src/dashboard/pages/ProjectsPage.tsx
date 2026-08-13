import { ArrowLeft, Clock, Database, FolderOpen, Lightning, Robot } from "@phosphor-icons/react";
import { useEffect, useState } from "react";
import { UsageTrend } from "../../components/charts/UsageTrend";
import { GlassCard } from "../../components/common/GlassCard";
import { MetricLegend } from "../../components/common/MetricLegend";
import { SourceBadge } from "../../components/common/SourceBadge";
import { getProjectDetail } from "../../services/analytics";
import type { AppSnapshot, ProjectDetail, ProjectUsage } from "../../types/analytics";
import { formatDuration, formatTokens } from "../../utils/format";
import { ProjectRanking } from "../components/ProjectRanking";

export function ProjectsPage({ snapshot, selected, onSelect, onBack }: { snapshot: AppSnapshot; selected: ProjectUsage | null; onSelect: (project: ProjectUsage) => void; onBack: () => void }) {
  if (!selected) return <ProjectRanking projects={snapshot.usage.projects} onSelect={onSelect} expanded />;
  return <ProjectDetailView selected={selected} onBack={onBack} />;
}

function ProjectDetailView({ selected, onBack }: { selected: ProjectUsage; onBack: () => void }) {
  const [detail, setDetail] = useState<ProjectDetail | null>(null);
  useEffect(() => { let active = true; void getProjectDetail(selected.id, "thirtyDays", selected).then((value) => { if (active) setDetail(value); }); return () => { active = false; }; }, [selected]);
  const summary = detail?.summary ?? selected;
  const source = summary.source;
  return <div className="detail-page">
    <button className="back-button" onClick={onBack}><ArrowLeft />返回项目排行</button>
    <GlassCard className="project-detail-hero"><div className="project-title-icon"><FolderOpen weight="fill" /></div><div className="project-title-copy"><SourceBadge source={source} /><h2>{summary.name}</h2><code title={summary.path}>{summary.path || "路径不可用"}</code></div><strong>{formatTokens(summary.total)}</strong></GlassCard>
    <div className="detail-metrics"><GlassCard><Robot weight="fill" /><span><small>Sessions</small><strong>{summary.sessions}</strong></span></GlassCard><GlassCard><Lightning weight="fill" /><span><small>Requests</small><strong>{summary.requests}</strong></span></GlassCard><GlassCard><Clock weight="fill" /><span><small>使用时长</small><strong>{formatDuration(summary.activeSeconds)}</strong></span></GlassCard><GlassCard><Database weight="fill" /><span><small>缓存命中</small><strong>{Math.round(summary.cached / Math.max(summary.input, 1) * 100)}%</strong></span></GlassCard></div>
    <div className="detail-grid"><GlassCard className="detail-chart"><div className="panel-header compact"><div><span className="panel-kicker">每日趋势 · {source}</span><h2>项目 Token 使用</h2></div></div><UsageTrend data={detail?.trend ?? []} /></GlassCard><GlassCard className="detail-breakdown"><div className="panel-header compact"><div><span className="panel-kicker">Token 构成</span><h2>输入与输出</h2></div></div><MetricLegend data={summary} /><dl><div><dt>平均每次 Session</dt><dd>{formatTokens(detail?.averageSessionTokens ?? (summary.sessions ? summary.total / summary.sessions : null))}</dd></div><div><dt>最后活动</dt><dd>{summary.lastActiveAt || "—"}</dd></div><div><dt>关联模型</dt><dd>{detail?.models.map((item) => item.name).join(" · ") || "—"}</dd></div><div><dt>关联 Skill</dt><dd>{detail?.skills.map((item) => item.name).join(" · ") || "—"}</dd></div></dl></GlassCard></div>
  </div>;
}
