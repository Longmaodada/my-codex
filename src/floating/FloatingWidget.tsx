import { GearSix, LockSimple, Power, SquaresFour, ArrowsClockwise, CloudCheck } from "@phosphor-icons/react";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useMemo, useRef, useState } from "react";
import { Heatmap } from "../components/charts/Heatmap";
import { QuotaRing } from "../components/charts/QuotaRing";
import { BrandMark } from "../components/common/BrandMark";
import { GlassCard } from "../components/common/GlassCard";
import { MetricLegend } from "../components/common/MetricLegend";
import { SourceBadge } from "../components/common/SourceBadge";
import { useCountdown } from "../hooks/useCountdown";
import { setWidgetCompact, windowAction } from "../services/backend";
import { useAppStore } from "../stores/appStore";
import { formatDateTime, formatTokens, percentage } from "../utils/format";

export function FloatingWidget() {
  const { snapshot, refreshing, refresh, updateSettings } = useAppStore();
  const [compact, setCompact] = useState(false);
  const idleTimer = useRef<number | null>(null);
  const quota = snapshot?.quota;
  const usage = snapshot?.usage;
  const settings = snapshot?.settings;
  const capsuleMode = settings?.capsuleMode ?? false;
  const selectedWindow = quota?.windows.find((item) => item.id === settings?.primaryQuotaWindow) ?? quota?.windows[0];
  const countdown = useCountdown(selectedWindow?.resetAt ?? null);
  const todayGoal = usage?.goals?.today ?? null;
  const weekGoal = usage?.goals?.week ?? null;
  const todayShare = todayGoal ? percentage(usage?.today.total ?? 0, todayGoal) : null;
  const weekShare = weekGoal ? percentage(usage?.week.total ?? 0, weekGoal) : null;
  const total90 = useMemo(() => usage?.daily.reduce((sum, day) => sum + day.total, 0) ?? 0, [usage?.daily]);

  const clearIdle = () => {
    if (idleTimer.current) window.clearTimeout(idleTimer.current);
    idleTimer.current = null;
  };

  const scheduleIdle = () => {
    clearIdle();
    if (!capsuleMode) return;
    idleTimer.current = window.setTimeout(() => {
      setCompact(true);
      void setWidgetCompact(true);
    }, 250);
  };

  useEffect(() => {
    scheduleIdle();
    return clearIdle;
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [capsuleMode]);

  useEffect(() => {
    if (!settings) return;
    setCompact(capsuleMode);
    void setWidgetCompact(capsuleMode);
  }, [capsuleMode]);

  const expand = () => {
    clearIdle();
    if (compact) {
      setCompact(false);
      void setWidgetCompact(false);
    }
  };

  if (!snapshot || !usage || !quota || !settings) return null;

  return (
    <motion.main
      className={`widget-shell ${compact ? "is-compact" : ""}`}
      initial={{ opacity: 0, scale: 0.96 }}
      animate={{ opacity: 1, scale: 1, width: compact ? 140 : 320, height: compact ? 56 : 500 }}
      transition={{ type: "spring", stiffness: 290, damping: 28 }}
      onMouseEnter={expand}
      onMouseLeave={capsuleMode ? scheduleIdle : undefined}
    >
      <div className="ambient ambient-one" />
      <div className="ambient ambient-two" />
      {compact ? (
        <button className="compact-content" onClick={expand} aria-label="展开 My Codex 悬浮窗">
          <BrandMark size={30} />
          <span><strong>{selectedWindow?.remainingPercent ?? "—"}%</strong><small>剩余</small></span>
        </button>
      ) : (
        <AnimatePresence mode="wait">
          <motion.div key="expanded" className="widget-content" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
            <header className="widget-header" data-tauri-drag-region={settings.lockPosition ? undefined : ""}>
              <div className="brand-lockup" data-tauri-drag-region={settings.lockPosition ? undefined : ""}>
                <BrandMark size={31} />
                <span><strong>My Codex</strong><small>{quota.plan ?? "未连接 Codex"}</small></span>
              </div>
              <div className="widget-actions">
                <button className={`icon-button ${settings.lockPosition ? "is-active" : ""}`} title={settings.lockPosition ? "解锁位置" : "锁定位置"} onClick={() => void updateSettings({ lockPosition: !settings.lockPosition })}><LockSimple weight={settings.lockPosition ? "fill" : "regular"} /></button>
                <span className="sync-state" title="数据仅在本机处理"><CloudCheck weight="fill" /></span>
                <button className={`icon-button ${refreshing ? "is-spinning" : ""}`} title="立即刷新" onClick={() => void refresh()} disabled={refreshing}><ArrowsClockwise /></button>
              </div>
            </header>

            {settings.mockMode && <div className="mock-banner"><SourceBadge source="mock" compact /><span>界面展示数据，不是账户额度</span></div>}

            <GlassCard className="widget-hero">
                <QuotaRing value={selectedWindow?.remainingPercent ?? null} size={98} stroke={9} sublabel={selectedWindow?.label ?? "额度"} />
              <div className="widget-today">
                <div className="section-eyebrow"><span>今日 Token</span><SourceBadge source={settings.mockMode ? "mock" : "local"} compact /></div>
                <strong>{formatTokens(usage.today.total)}</strong>
                <MetricLegend data={usage.today} compact />
                {todayShare !== null && <div className="thin-progress"><i style={{ width: `${todayShare}%` }} /></div>}
              </div>
            </GlassCard>

            <GlassCard className="widget-targets">
              <div className="card-heading"><span>Token 小目标</span><small>本地统计</small></div>
              <div className="target-row"><span>今日</span><strong>{formatTokens(usage.today.total)}{todayGoal ? ` / ${formatTokens(todayGoal)}` : ""}</strong><em>{todayShare === null ? "—" : `${todayShare}%`}</em></div>
              {todayShare !== null && <div className="thin-progress"><i style={{ width: `${todayShare}%` }} /></div>}
              <div className="target-row"><span>本周</span><strong>{formatTokens(usage.week.total)}{weekGoal ? ` / ${formatTokens(weekGoal)}` : ""}</strong><em>{weekShare === null ? "—" : `${weekShare}%`}</em></div>
              {weekShare !== null && <div className="thin-progress muted"><i style={{ width: `${weekShare}%` }} /></div>}
              <div className="reset-row">
                <span><small>下一次重置</small><strong>{formatDateTime(selectedWindow?.resetAt ?? null)}</strong></span>
                <em>{countdown}</em>
              </div>
            </GlassCard>

            <GlassCard className="widget-heat">
              <div className="card-heading"><span>近 90 天用量</span><SourceBadge source={settings.mockMode ? "mock" : "local"} compact /></div>
              <Heatmap data={usage.daily} weeks={13} />
              <div className="heat-total"><span>近 90 天累计</span><strong>{formatTokens(total90)} <small>Token</small></strong></div>
            </GlassCard>

            <footer className="widget-footer">
              <button onClick={() => void windowAction("show_dashboard")}><SquaresFour weight="fill" />打开主面板</button>
              <button onClick={() => void windowAction("open_settings")}><GearSix />设置</button>
              <button onClick={() => void windowAction("quit")}><Power />退出</button>
            </footer>
          </motion.div>
        </AnimatePresence>
      )}
    </motion.main>
  );
}
