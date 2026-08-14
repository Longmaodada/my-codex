import { Lightning, LinkSimple, Sparkle } from "@phosphor-icons/react";
import { GlassCard } from "../../components/common/GlassCard";
import { SourceBadge } from "../../components/common/SourceBadge";
import type { SkillUsage } from "../../types/analytics";
import { formatTokens, percentage } from "../../utils/format";

export function SkillsPage({ skills }: { skills: SkillUsage[] }) {
  const hasTokenUsage = skills.some((skill) => skill.total > 0);
  const rankedSkills = [...skills].sort((left, right) => {
    const leftMetric = hasTokenUsage ? left.total : left.invocations;
    const rightMetric = hasTokenUsage ? right.total : right.invocations;
    return rightMetric - leftMetric || right.invocations - left.invocations || left.name.localeCompare(right.name);
  });
  const max = Math.max(...rankedSkills.map((skill) => hasTokenUsage ? skill.total : skill.invocations), 1);
  const source = rankedSkills[0]?.source ?? "estimated";
  return (
    <GlassCard className="analytics-table-card">
      <div className="panel-header"><div><span className="panel-kicker"><Sparkle weight="fill" />Local Usage Analytics</span><h2>Skill 用量排行</h2></div><SourceBadge source={source} /></div>
      <p className="panel-note">调用次数按明确记录的 Skill 事件统计；Token、缓存命中和平均调用仅在本页按同一会话内的调用次数加权归属。归属置信度反映同会话中可区分程度。</p>
      <div className="analytics-list">
        {rankedSkills.map((skill, index) => (
          <div className="analytics-row" key={skill.name}>
            <span className="analytics-rank">{String(index + 1).padStart(2, "0")}</span>
            <span className="skill-icon"><Lightning weight="fill" /></span>
            <span className="analytics-main"><strong>{skill.name}</strong><small><LinkSimple />{skill.projects.join(" · ") || "本地会话"}</small><i><em style={{ width: `${percentage(hasTokenUsage ? skill.total : skill.invocations, max)}%` }} /></i></span>
            <span><small>调用</small><strong>{skill.invocations}</strong></span>
            <span><small>Token</small><strong className={skill.total > 0 ? undefined : "metric-unavailable"}>{skill.total > 0 ? formatTokens(skill.total) : "—"}</strong></span>
            <span><small>缓存命中</small><strong className={skill.cacheHitRatio === null ? "metric-unavailable" : undefined}>{skill.cacheHitRatio === null ? "—" : `${Math.round(skill.cacheHitRatio)}%`}</strong></span>
            <span><small>归属置信度</small><strong className={skill.confidence === null ? "metric-unavailable" : undefined}>{skill.confidence === null ? "—" : `${Math.round(skill.confidence * 100)}%`}</strong></span>
            <span><small>平均调用</small><strong className={skill.total > 0 && skill.invocations > 0 ? undefined : "metric-unavailable"}>{skill.total > 0 && skill.invocations > 0 ? formatTokens(skill.total / skill.invocations) : "—"}</strong></span>
          </div>
        ))}
      </div>
    </GlassCard>
  );
}
