import { Lightning, LinkSimple, Sparkle } from "@phosphor-icons/react";
import { GlassCard } from "../../components/common/GlassCard";
import { SourceBadge } from "../../components/common/SourceBadge";
import type { SkillUsage } from "../../types/analytics";
import { formatTokens, percentage } from "../../utils/format";

export function SkillsPage({ skills }: { skills: SkillUsage[] }) {
  const max = skills[0]?.total ?? 1;
  const source = skills[0]?.source ?? "estimated";
  return (
    <GlassCard className="analytics-table-card">
      <div className="panel-header"><div><span className="panel-kicker"><Sparkle weight="fill" />Local Usage Analytics</span><h2>Skill 用量排行</h2></div><SourceBadge source={source} /></div>
      <p className="panel-note">Token 通过本地会话事件与明确的 Skill 调用信号关联估算，并非官方账户账单。只出现于“可用 Skill 列表”不会计为调用。</p>
      <div className="analytics-list">
        {skills.map((skill, index) => (
          <div className="analytics-row" key={skill.name}>
            <span className="analytics-rank">{String(index + 1).padStart(2, "0")}</span>
            <span className="skill-icon"><Lightning weight="fill" /></span>
            <span className="analytics-main"><strong>{skill.name}</strong><small><LinkSimple />{skill.projects.join(" · ")}</small><i><em style={{ width: `${percentage(skill.total, max)}%` }} /></i></span>
            <span><small>调用</small><strong>{skill.invocations}</strong></span>
            <span><small>Token</small><strong>{formatTokens(skill.total)}</strong></span>
            <span><small>置信度</small><strong>{Math.round(skill.confidence * 100)}%</strong></span>
            <span><small>平均调用</small><strong>{formatTokens(skill.total / skill.invocations)}</strong></span>
          </div>
        ))}
      </div>
    </GlassCard>
  );
}
