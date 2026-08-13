import { CalendarBlank, ClockCountdown, Infinity as InfinityIcon, Sparkle } from "@phosphor-icons/react";
import { motion } from "framer-motion";
import { QuotaRing } from "../../components/charts/QuotaRing";
import { GlassCard } from "../../components/common/GlassCard";
import { MetricLegend } from "../../components/common/MetricLegend";
import { SourceBadge } from "../../components/common/SourceBadge";
import { useCountdown } from "../../hooks/useCountdown";
import type { AppSnapshot, QuotaWindow, TokenBreakdown } from "../../types/analytics";
import { formatTokens } from "../../utils/format";

function TokenCard({ title, data, icon, source }: { title: string; data: TokenBreakdown; icon: React.ReactNode; source: "local" | "mock" }) {
  return (
    <GlassCard className="summary-card token-summary-card">
      <div className="summary-label"><span>{icon}{title}</span><SourceBadge source={source} compact /></div>
      <strong className="summary-number">{formatTokens(data.total)}</strong>
      <MetricLegend data={data} compact />
      <div className="summary-progress"><span style={{ width: `${Math.min(100, Math.max(18, (data.cached / Math.max(data.input, 1)) * 100))}%` }} /></div>
    </GlassCard>
  );
}

function QuotaCard({ quota, source }: { quota: QuotaWindow | undefined; source: "official" | "mock" }) {
  const countdown = useCountdown(quota?.resetAt ?? null);
  return (
    <GlassCard className="summary-card quota-summary-card">
      <QuotaRing value={quota?.remainingPercent ?? null} size={126} stroke={10} sublabel={quota?.label ?? "额度"} />
      <div className="quota-summary-copy">
        <div className="summary-label"><span><Sparkle weight="fill" />额度</span><SourceBadge source={source} compact /></div>
        <strong>{quota?.label ?? "数据不可用"}</strong>
        <small><ClockCountdown /> {countdown}</small>
      </div>
    </GlassCard>
  );
}

export function SummaryCards({ snapshot }: { snapshot: AppSnapshot }) {
  const quota = snapshot.quota.windows.find((item) => item.id === snapshot.settings.primaryQuotaWindow) ?? snapshot.quota.windows[0];
  const source = snapshot.settings.mockMode ? "mock" : "local";
  return (
    <motion.div className="summary-grid" initial="hidden" animate="show" variants={{ show: { transition: { staggerChildren: 0.07 } } }}>
      <QuotaCard quota={quota} source={snapshot.settings.mockMode ? "mock" : "official"} />
      <TokenCard title="今日" data={snapshot.usage.today} source={source} icon={<CalendarBlank weight="fill" />} />
      <TokenCard title="近 7 天" data={snapshot.usage.last7Days} source={source} icon={<ClockCountdown weight="fill" />} />
      <TokenCard title="累计" data={snapshot.usage.lifetime} source={source} icon={<InfinityIcon weight="bold" />} />
    </motion.div>
  );
}
