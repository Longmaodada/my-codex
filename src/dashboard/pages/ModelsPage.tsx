import { Robot } from "@phosphor-icons/react";
import { GlassCard } from "../../components/common/GlassCard";
import { SourceBadge } from "../../components/common/SourceBadge";
import type { ModelUsage } from "../../types/analytics";
import { formatTokens, percentage } from "../../utils/format";

export function ModelsPage({ models }: { models: ModelUsage[] }) {
  const total = models.reduce((sum, model) => sum + model.total, 0);
  const source = models[0]?.source ?? "local";
  return (
    <GlassCard className="analytics-table-card">
      <div className="panel-header"><div><span className="panel-kicker"><Robot weight="fill" />本地 Session 识别</span><h2>模型用量</h2></div><SourceBadge source={source} /></div>
      <div className="model-grid">
        {models.map((model, index) => (
          <div className="model-card" key={model.name}>
            <span className={`model-orb model-orb-${index + 1}`}><Robot weight="fill" /></span>
            <span><small>模型</small><strong>{model.name}</strong></span>
            <span><small>Token</small><strong>{formatTokens(model.total)}</strong></span>
            <span><small>请求</small><strong>{model.requests}</strong></span>
            <span><small>占比</small><strong>{percentage(model.total, total)}%</strong></span>
            <span><small>缓存命中</small><strong>{model.cacheHitRatio}%</strong></span>
          </div>
        ))}
      </div>
    </GlassCard>
  );
}
