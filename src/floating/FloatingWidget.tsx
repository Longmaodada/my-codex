import { GearSix, LockSimple, Power, SquaresFour, ArrowsClockwise, CloudCheck } from "@phosphor-icons/react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Info, Ticket, WarningCircle } from "@phosphor-icons/react";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useMemo, useRef, useState } from "react";
import { Heatmap } from "../components/charts/Heatmap";
import { QuotaArc, QuotaRing } from "../components/charts/QuotaRing";
import { BrandMark } from "../components/common/BrandMark";
import { GlassCard } from "../components/common/GlassCard";
import { MetricLegend } from "../components/common/MetricLegend";
import { SourceBadge } from "../components/common/SourceBadge";
import { summarizeResetVouchers } from "../dashboard/components/ResetVoucherCard";
import { useCountdown } from "../hooks/useCountdown";
import { isTauri, setWidgetCompact, windowAction } from "../services/backend";
import { useAppStore } from "../stores/appStore";
import { formatDateTime, formatTokens, percentage } from "../utils/format";

export function FloatingWidget() {
  const { snapshot, refreshing, refresh, updateSettings } = useAppStore();
  const [compact, setCompact] = useState(() => {
    if (snapshot?.settings.lockPosition) return false;
    return snapshot?.settings.capsuleMode ?? false;
  });
  const compactTarget = useRef(compact);
  const lockPositionRef = useRef(snapshot?.settings.lockPosition ?? false);
  const quota = snapshot?.quota;
  const usage = snapshot?.usage;
  const settings = snapshot?.settings;
  const capsuleMode = settings?.capsuleMode ?? false;
  const lockPosition = settings?.lockPosition ?? false;
  const selectedWindow = quota?.windows.find((item) => item.id === settings?.primaryQuotaWindow) ?? quota?.windows[0];
  const remainingPercent = selectedWindow?.remainingPercent ?? null;
  const remainingLabel = remainingPercent === null ? "—" : `${Math.round(remainingPercent)}%`;
  const countdown = useCountdown(selectedWindow?.resetAt ?? null);
  const todayGoal = usage?.goals?.today ?? null;
  const todayShare = todayGoal ? percentage(usage?.today.total ?? 0, todayGoal) : null;
  const voucherSummary = summarizeResetVouchers(snapshot?.officialResetVouchers);
  const total90 = useMemo(() => usage?.daily.reduce((sum, day) => sum + day.total, 0) ?? 0, [usage?.daily]);

  useEffect(() => {
    const wasLocked = lockPositionRef.current;
    lockPositionRef.current = lockPosition;

    if (lockPosition) {
      compactTarget.current = false;
      setCompact(false);
      void setWidgetCompact(false);
      return;
    }

    // Unlocking keeps the expanded My Codex surface until the pointer leaves it.
    // This avoids an immediate jump back to the capsule while the unlock button
    // is still under the pointer.
    if (wasLocked) return;

    compactTarget.current = capsuleMode;
    setCompact(capsuleMode);
    void setWidgetCompact(capsuleMode);
  }, [capsuleMode, lockPosition]);

  const requestCompact = (next: boolean) => {
    if (!capsuleMode || lockPosition || compactTarget.current === next) return;
    compactTarget.current = next;
    setCompact(next);
    void setWidgetCompact(next).then(() => {
      if (compactTarget.current !== next) void setWidgetCompact(compactTarget.current);
    });
  };

  const startWindowDrag = (event: React.PointerEvent<HTMLElement>) => {
    if (event.button !== 0 || !isTauri() || settings?.lockPosition) return;
    if ((event.target as HTMLElement).closest("button, a, input, select")) return;
    void getCurrentWindow().startDragging();
  };

  if (!snapshot || !usage || !quota || !settings) return null;

  return (
    <motion.main
      className={`widget-shell ${compact ? "is-compact" : ""}`}
      initial={{ opacity: 0, scale: 0.96 }}
      animate={{ opacity: 1, scale: 1, width: compact ? 140 : 320, height: compact ? 56 : 500 }}
      transition={{ type: "spring", stiffness: 290, damping: 28 }}
      onPointerEnter={() => requestCompact(false)}
      onPointerLeave={() => requestCompact(true)}
      onPointerDown={startWindowDrag}
    >
      <div className="ambient ambient-one" />
      <div className="ambient ambient-two" />
      {compact ? (
        <motion.div className="compact-content" aria-label="展开 My Codex 悬浮窗" initial={{ opacity: 0, scale: 0.82 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0, scale: 0.82 }} transition={{ type: "spring", stiffness: 360, damping: 26 }}>
          <span className="compact-quota-copy"><strong>{remainingLabel}</strong><small>剩余额度</small></span>
          <span className="compact-meter"><QuotaArc value={remainingPercent} size={42} stroke={4} /></span>
        </motion.div>
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

            {settings.mockMode && <div className="mock-banner"><SourceBadge source="mock" compact /><span>界面展示数据，仅用于验证完整界面</span></div>}

            <GlassCard className="widget-hero">
                <QuotaRing value={selectedWindow?.remainingPercent ?? null} size={98} stroke={9} sublabel={selectedWindow?.label ?? "额度"} />
              <div className="widget-today">
                <div className="section-eyebrow"><span>今日 Token</span><SourceBadge source={settings.mockMode ? "mock" : "local"} compact /></div>
                <strong>{formatTokens(usage.today.total)}</strong>
                <MetricLegend data={usage.today} compact />
                {todayShare !== null && <div className="thin-progress"><i style={{ transform: `scaleX(${todayShare / 100})` }} /></div>}
              </div>
            </GlassCard>

            <GlassCard className="widget-voucher">
              <div className="card-heading"><span><Ticket aria-hidden="true" weight="fill" />Codex 额度重置卷</span><small>官方发放</small></div>
              <div className="voucher-summary">
                <div><small>持有总数</small><strong>{voucherSummary.total === null ? "—" : voucherSummary.total}<em>张</em></strong></div>
                <div><small>当前可使用</small><strong>{voucherSummary.available === null ? "—" : voucherSummary.available}<em>张</em></strong></div>
              </div>
              <div className={`voucher-note ${voucherSummary.unavailable ? "is-warning" : ""}`} role={voucherSummary.unavailable ? "status" : undefined}>
                {voucherSummary.unavailable ? <WarningCircle aria-hidden="true" weight="fill" /> : <Info aria-hidden="true" weight="fill" />}
                <span>{voucherSummary.unavailable ? "获取失败：官方重置卷信息暂不可用，请稍后刷新。" : "官方重置卷仅用于额度重置，不会删除 My Codex 的本地统计、设置或任何软件数据。"}</span>
              </div>
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

