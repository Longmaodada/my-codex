import { GearSix, LockSimple, Power, SquaresFour, ArrowsClockwise, CloudCheck } from "@phosphor-icons/react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useMemo, useRef, useState } from "react";
import { Heatmap } from "../components/charts/Heatmap";
import { QuotaRing } from "../components/charts/QuotaRing";
import { BrandMark } from "../components/common/BrandMark";
import { GlassCard } from "../components/common/GlassCard";
import { MetricLegend } from "../components/common/MetricLegend";
import { SourceBadge } from "../components/common/SourceBadge";
import { useCountdown } from "../hooks/useCountdown";
import { isTauri, setWidgetCompact, windowAction } from "../services/backend";
import { useAppStore } from "../stores/appStore";
import { formatDateTime, formatTokens, percentage } from "../utils/format";

export function FloatingWidget() {
  const { snapshot, refreshing, refresh, updateSettings } = useAppStore();
  const [compact, setCompact] = useState(false);
  const idleTimer = useRef<number | null>(null);
  const hoverTimer = useRef<number | null>(null);
  const capsulePointer = useRef<{ x: number; y: number; moved: boolean } | null>(null);
  const quota = snapshot?.quota;
  const usage = snapshot?.usage;
  const settings = snapshot?.settings;
  const capsuleMode = settings?.capsuleMode ?? false;
  const selectedWindow = quota?.windows.find((item) => item.id === settings?.primaryQuotaWindow) ?? quota?.windows[0];
  const remainingPercent = selectedWindow?.remainingPercent ?? null;
  const remainingLabel = remainingPercent === null ? "—" : `${Math.round(remainingPercent)}%`;
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

  const clearHover = () => {
    if (hoverTimer.current) window.clearTimeout(hoverTimer.current);
    hoverTimer.current = null;
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
    return () => { clearIdle(); clearHover(); };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [capsuleMode]);

  useEffect(() => {
    if (!settings) return;
    setCompact(capsuleMode);
    clearHover();
    void setWidgetCompact(capsuleMode);
  }, [capsuleMode]);

  useEffect(() => () => {
    clearIdle();
    clearHover();
  }, []);

  const expand = () => {
    clearIdle();
    capsulePointer.current = null;
    if (compact) {
      setCompact(false);
      void setWidgetCompact(false);
    }
  };

  const scheduleExpand = () => {
    if (!compact || capsulePointer.current) return;
    clearHover();
    hoverTimer.current = window.setTimeout(expand, 140);
  };

  const handleCapsulePointerDown = (event: React.PointerEvent<HTMLButtonElement>) => {
    if (settings?.lockPosition) return;
    capsulePointer.current = { x: event.clientX, y: event.clientY, moved: false };
    if (isTauri()) void getCurrentWindow().startDragging();
  };

  const handleCapsulePointerMove = (event: React.PointerEvent<HTMLButtonElement>) => {
    const pointer = capsulePointer.current;
    if (!pointer) return;
    pointer.moved = pointer.moved || Math.hypot(event.clientX - pointer.x, event.clientY - pointer.y) > 5;
  };

  const handleCapsuleClick = () => {
    const moved = capsulePointer.current?.moved ?? false;
    capsulePointer.current = null;
    if (!moved) expand();
  };

  const handleCapsuleDoubleClick = () => {
    capsulePointer.current = null;
    expand();
  };

  if (!snapshot || !usage || !quota || !settings) return null;

  return (
    <motion.main
      className={`widget-shell ${compact ? "is-compact" : ""}`}
      data-tauri-drag-region={!compact && !settings.lockPosition ? "" : undefined}
      initial={{ opacity: 0, scale: 0.96 }}
      animate={{ opacity: 1, scale: 1, width: compact ? 140 : 320, height: compact ? 56 : 500 }}
      transition={{ type: "spring", stiffness: 290, damping: 28 }}
      onPointerDown={compact ? handleCapsulePointerDown : undefined}
      onPointerMove={compact ? handleCapsulePointerMove : undefined}
      onPointerEnter={compact ? scheduleExpand : undefined}
      onPointerLeave={capsuleMode ? scheduleIdle : undefined}
    >
      <div className="ambient ambient-one" />
      <div className="ambient ambient-two" />
      {compact ? (
        <button className="compact-content" data-tauri-drag-region={undefined} onClick={handleCapsuleClick} onDoubleClick={handleCapsuleDoubleClick} aria-label="展开 My Codex 悬浮窗">
          <span className="compact-quota-copy"><strong>{remainingLabel}</strong><small>剩余额度</small></span>
          <span className="compact-meter" aria-hidden="true">
            <svg viewBox="0 0 42 42" role="presentation">
              <circle className="compact-meter-track" cx="21" cy="21" r="16" />
              <circle className={`compact-meter-value ${(remainingPercent ?? 0) <= 20 ? "is-low" : ""}`} cx="21" cy="21" r="16" pathLength="100" style={{ strokeDasharray: `${Math.max(0, Math.min(100, remainingPercent ?? 0))} 100` }} />
            </svg>
          </span>
        </button>
      ) : (
        <AnimatePresence mode="wait">
          <motion.div key="expanded" className="widget-content" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
            <header className="widget-header">
              <div className="brand-lockup">
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
                <span><small>剩余额度 · 下一次重置</small><strong>{remainingLabel} · {formatDateTime(selectedWindow?.resetAt ?? null)}</strong></span>
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
