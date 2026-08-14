import { CaretRight, CrownSimple, FolderSimple } from "@phosphor-icons/react";
import { useEffect, useState } from "react";
import { GlassCard } from "../../components/common/GlassCard";
import type { ProjectUsage } from "../../types/analytics";
import { formatDuration, formatTokens, percentage } from "../../utils/format";
import { getProjects } from "../../services/analytics";
import type { UsageRange } from "../../types/analytics";

const rangeLabels: Record<UsageRange, string> = { today: "今天", sevenDays: "7天", thirtyDays: "30天", ninetyDays: "90天", all: "全部" };

export function ProjectRanking({ projects, onSelect, expanded = false, initialRange = "sevenDays" }: { projects: ProjectUsage[]; onSelect: (project: ProjectUsage) => void; expanded?: boolean; initialRange?: UsageRange }) {
  const [range, setRange] = useState(rangeLabels[initialRange]);
  const [visibleProjects, setVisibleProjects] = useState(projects);
  const maximum = visibleProjects[0]?.total ?? 1;
  const visible = expanded ? visibleProjects : visibleProjects.slice(0, 6);
  const ranges: Record<string, UsageRange> = { "今天": "today", "7天": "sevenDays", "30天": "thirtyDays", "全部": "all" };

  useEffect(() => {
    let active = true;
    setRange(rangeLabels[initialRange]);
    setVisibleProjects(projects);
    void getProjects(initialRange, 50, projects).then((nextProjects) => {
      if (active) setVisibleProjects(nextProjects);
    });
    return () => { active = false; };
  }, [initialRange, projects]);
  const chooseRange = (label: string) => {
    setRange(label);
    void getProjects(ranges[label], 50, projects).then(setVisibleProjects);
  };
  return (
    <GlassCard className={`ranking-card ${expanded ? "ranking-expanded" : ""}`}>
      <div className="panel-header">
        <div><span className="panel-kicker"><FolderSimple weight="fill" />本地统计</span><h2>项目用量排行</h2></div>
        <div className="segmented compact-segmented">
          {["今天", "7天", "30天", "全部"].map((item) => <button key={item} className={range === item ? "active" : ""} onClick={() => chooseRange(item)}>{item}</button>)}
        </div>
      </div>
      <div className="ranking-list">
        {visible.map((project, index) => (
          <button className="ranking-row" key={project.id} onClick={() => onSelect(project)}>
            <span className={`rank-index rank-${index + 1}`}>{index < 3 ? <CrownSimple weight="fill" /> : index + 1}</span>
            <span className="rank-main">
              <span className="rank-title"><strong>{project.name}</strong><em>{formatTokens(project.total)}</em></span>
              <span className="rank-meta">{project.sessions} 会话 · {formatDuration(project.activeSeconds)} · {project.lastActiveAt}</span>
              <span className="rank-progress"><i style={{ width: `${percentage(project.total, maximum)}%` }} /></span>
            </span>
            <CaretRight className="row-arrow" />
          </button>
        ))}
      </div>
    </GlassCard>
  );
}
