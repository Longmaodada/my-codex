import { Database, Gauge, Leaf, TrendUp } from "@phosphor-icons/react";
import { UsageTrend } from "../../components/charts/UsageTrend";
import { GlassCard } from "../../components/common/GlassCard";
import { SourceBadge } from "../../components/common/SourceBadge";
import type { AppSnapshot } from "../../types/analytics";
import { formatTokens } from "../../utils/format";

export function CachePage({ snapshot }: { snapshot: AppSnapshot }) {
  const cached = snapshot.usage.lifetime.cached;
  const miss = Math.max(snapshot.usage.lifetime.input - cached, 0);
  const saved = Math.round(cached * 0.78);
  const source = snapshot.settings.mockMode ? "mock" : "local";
  const savedSource = snapshot.settings.mockMode ? "mock" : "estimated";
  return (
    <div className="cache-page">
      <div className="cache-metric-grid">
        <GlassCard><Database weight="fill" /><span><small>缓存 Token</small><strong>{formatTokens(cached)}</strong></span><SourceBadge source={source} compact /></GlassCard>
        <GlassCard><Gauge weight="fill" /><span><small>缓存命中率</small><strong>{snapshot.usage.cacheHitRatio}%</strong></span><SourceBadge source={source} compact /></GlassCard>
        <GlassCard><TrendUp weight="fill" /><span><small>未命中 Input</small><strong>{formatTokens(miss)}</strong></span><SourceBadge source={source} compact /></GlassCard>
        <GlassCard><Leaf weight="fill" /><span><small>估算节省</small><strong>{formatTokens(saved)}</strong></span><SourceBadge source={savedSource} compact /></GlassCard>
      </div>
      <GlassCard className="cache-chart-card"><div className="panel-header"><div><span className="panel-kicker">Cache Read · 本地统计</span><h2>缓存命中趋势</h2></div></div><UsageTrend data={snapshot.usage.daily.slice(-30).map((day) => ({ ...day, total: day.cached }))} /></GlassCard>
      <GlassCard className="cache-explanation"><strong>计算口径</strong><p>Cached Input 是 Input 的子集，总 Token 直接使用事件中的 total_tokens，不会把缓存与 reasoning 再次相加。未命中 Input = max(Input - Cached Input, 0)；“估算节省”只作本地推算，不代表官方费用账单。</p></GlassCard>
    </div>
  );
}
