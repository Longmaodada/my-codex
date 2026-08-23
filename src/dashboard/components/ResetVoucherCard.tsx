import {
  CalendarDots,
  CheckCircle,
  Clock,
  Info,
  Ticket,
  WarningCircle,
  XCircle,
} from "@phosphor-icons/react";
import type { HTMLMotionProps } from "framer-motion";
import { GlassCard } from "../../components/common/GlassCard";
import type { OfficialResetVoucher } from "../../types/analytics";

export type ResetVoucherStatus = OfficialResetVoucher["status"];
export type ResetVoucher = Pick<OfficialResetVoucher, "id" | "expiresAt" | "status" | "label">;

export const OFFICIAL_RESET_VOUCHER_UNAVAILABLE_MESSAGE = "获取官方重置卷失败，暂时无法获取信息。";

export interface ResetVoucherSummary {
  total: number | null;
  available: number | null;
  unavailable: boolean;
}

export function summarizeResetVouchers(vouchers: readonly ResetVoucher[] | null | undefined): ResetVoucherSummary {
  if (vouchers == null) return { total: null, available: null, unavailable: true };
  return {
    total: vouchers.length,
    available: vouchers.filter((voucher) => voucher.status === "available").length,
    unavailable: false,
  };
}

export interface ResetVoucherCardProps extends Omit<HTMLMotionProps<"section">, "children"> {
  vouchers: readonly ResetVoucher[];
  reportedAvailableCount?: number;
  unavailable?: boolean;
  unavailableMessage?: string;
}

const statusCopy: Record<ResetVoucherStatus, string> = {
  available: "可使用",
  used: "已使用",
  expired: "已过期",
  unavailable: "不可用",
};

const statusIcon = {
  available: CheckCircle,
  used: Clock,
  expired: XCircle,
  unavailable: WarningCircle,
} satisfies Record<ResetVoucherStatus, typeof CheckCircle>;

const statusColor: Record<ResetVoucherStatus, string> = {
  available: "#278c67",
  used: "var(--text-muted)",
  expired: "#c95868",
  unavailable: "#b68122",
};

function formatExpiry(expiresAt: string | null | undefined) {
  if (!expiresAt) return "到期日期不可用";
  const date = new Date(expiresAt);
  if (Number.isNaN(date.getTime())) return "到期日期不可用";
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "long",
    day: "numeric",
    timeZone: "UTC",
  }).format(date);
}

function VoucherStatus({ status }: { status: ResetVoucherStatus }) {
  const Icon = statusIcon[status];
  return (
    <span
      style={{
        alignItems: "center",
        color: statusColor[status],
        display: "inline-flex",
        fontSize: 11,
        fontWeight: 700,
        gap: 4,
        whiteSpace: "nowrap",
      }}
    >
      <Icon aria-hidden="true" size={13} weight="fill" />
      {statusCopy[status]}
    </span>
  );
}

function EmptyState({ unavailable, message }: { unavailable: boolean; message: string }) {
  const Icon = unavailable ? WarningCircle : Ticket;
  return (
    <div
      role={unavailable ? "alert" : undefined}
      style={{
        alignItems: "center",
        color: "var(--text-muted)",
        display: "grid",
        gap: 6,
        justifyItems: "center",
        minHeight: 104,
        padding: "14px 8px 6px",
        textAlign: "center",
      }}
    >
      <Icon aria-hidden="true" color={unavailable ? "#b68122" : "var(--purple)"} size={24} />
      <strong style={{ color: "var(--text-secondary)", fontSize: 12 }}>{unavailable ? "获取失败：官方重置卷暂不可用" : "当前没有官方重置卷"}</strong>
      <small style={{ fontSize: 10, lineHeight: 1.5 }}>{message}</small>
      {unavailable && <small style={{ color: "var(--text-secondary)", fontSize: 10, fontWeight: 650 }}>下一步：稍后刷新。</small>}
    </div>
  );
}

/**
 * Displays official Codex reset vouchers only. It intentionally has no reset
 * action: this is an account-issued voucher inventory, not a local data reset.
 */
export function ResetVoucherCard({
  vouchers,
  reportedAvailableCount,
  unavailable = false,
  unavailableMessage = OFFICIAL_RESET_VOUCHER_UNAVAILABLE_MESSAGE,
  className = "",
  ...cardProps
}: ResetVoucherCardProps) {
  const availableCount = reportedAvailableCount ?? vouchers.filter((voucher) => voucher.status === "available").length;
  const emptyMessage = unavailable ? unavailableMessage : "当官方重置卷到账后，会在这里显示每张卷的到期日期。";

  return (
    <GlassCard
      {...cardProps}
      aria-label="官方重置卷"
      className={`reset-voucher-card ${className}`.trim()}
      style={{ padding: 17, ...cardProps.style }}
    >
      <div style={{ alignItems: "flex-start", display: "flex", gap: 12, justifyContent: "space-between" }}>
        <div style={{ display: "flex", gap: 10, minWidth: 0 }}>
          <span
            aria-hidden="true"
            style={{
              alignItems: "center",
              background: "rgba(103, 88, 237, .11)",
              borderRadius: 12,
              color: "var(--purple)",
              display: "inline-flex",
              flex: "none",
              height: 36,
              justifyContent: "center",
              width: 36,
            }}
          >
            <Ticket size={19} weight="fill" />
          </span>
          <div style={{ display: "grid", gap: 3, minWidth: 0 }}>
            <span style={{ alignItems: "center", color: "var(--text-secondary)", display: "inline-flex", fontSize: 10, fontWeight: 650, gap: 5 }}>
              <Ticket aria-hidden="true" size={12} weight="fill" />
              官方重置卷
            </span>
            <h2 style={{ fontSize: 16, letterSpacing: "-.35px", margin: 0 }}>Codex 额度重置卷</h2>
          </div>
        </div>
        <span
          style={{
            alignItems: "center",
            background: unavailable ? "rgba(229, 179, 68, .12)" : "rgba(103, 88, 237, .09)",
            borderRadius: 999,
            color: unavailable ? "#9d6b1e" : "#6655de",
            display: "inline-flex",
            flex: "none",
            fontSize: 10,
            fontWeight: 700,
            gap: 4,
            padding: "5px 8px",
          }}
        >
          <Info aria-hidden="true" size={12} weight="fill" />
          官方发放
        </span>
      </div>

      <div
        style={{
          alignItems: "center",
          background: "rgba(255, 255, 255, .22)",
          border: "1px solid rgba(103, 88, 237, .1)",
          borderRadius: 12,
          display: "flex",
          gap: 16,
          margin: "14px 0 12px",
          padding: "10px 12px",
        }}
      >
        <div style={{ display: "grid", gap: 2, minWidth: 88 }}>
          <span style={{ color: "var(--text-muted)", fontSize: 10 }}>持有总数</span>
          <strong style={{ fontSize: 25, letterSpacing: "-1px", lineHeight: 1 }}>{unavailable ? "—" : vouchers.length}<small style={{ color: "var(--text-secondary)", fontSize: 11, marginLeft: 4 }}>张</small></strong>
        </div>
        <div style={{ borderLeft: "1px solid rgba(94, 81, 132, .12)", display: "grid", gap: 2, paddingLeft: 16 }}>
          <span style={{ color: "var(--text-muted)", fontSize: 10 }}>当前可使用</span>
          <strong style={{ fontSize: 16, lineHeight: 1 }}>{unavailable ? "—" : `${availableCount} 张`}</strong>
        </div>
      </div>

      <div
        style={{
          alignItems: "flex-start",
          background: "rgba(83, 102, 239, .07)",
          borderRadius: 10,
          color: "var(--text-secondary)",
          display: "flex",
          fontSize: 10,
          gap: 7,
          lineHeight: 1.55,
          marginBottom: 12,
          padding: "8px 10px",
        }}
      >
        <Info aria-hidden="true" color="var(--blue)" size={14} style={{ flex: "none" }} weight="fill" />
        <span><strong>这不是软件数据重置。</strong> 官方重置卷由 Codex 官方发放，用于额度重置；不会删除 My Codex 的本地统计、设置或任何软件数据。</span>
      </div>

      {unavailable || vouchers.length === 0 ? (
        <EmptyState unavailable={unavailable} message={emptyMessage} />
      ) : (
        <div style={{ display: "grid" }}>
          {vouchers.map((voucher, index) => (
            <article
              key={voucher.id}
              style={{
                alignItems: "center",
                borderBottom: index === vouchers.length - 1 ? 0 : "1px solid rgba(94, 81, 132, .08)",
                display: "grid",
                gap: 10,
                gridTemplateColumns: "26px minmax(0, 1fr) auto",
                minHeight: 54,
                padding: "7px 2px",
              }}
            >
              <span
                aria-hidden="true"
                style={{
                  alignItems: "center",
                  background: "rgba(103, 88, 237, .08)",
                  borderRadius: 8,
                  color: "var(--purple)",
                  display: "inline-flex",
                  fontSize: 10,
                  fontWeight: 750,
                  height: 26,
                  justifyContent: "center",
                  width: 26,
                }}
              >
                {String(index + 1).padStart(2, "0")}
              </span>
              <div style={{ display: "grid", gap: 3, minWidth: 0 }}>
                <strong style={{ fontSize: 11, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={voucher.label ?? voucher.id}>
                  {voucher.label ?? `官方重置卷 ${index + 1}`}
                </strong>
                <span style={{ alignItems: "center", color: "var(--text-muted)", display: "inline-flex", fontSize: 9, gap: 4 }}>
                  <CalendarDots aria-hidden="true" size={11} />
                  到期：{formatExpiry(voucher.expiresAt)}
                </span>
              </div>
              <VoucherStatus status={voucher.status} />
            </article>
          ))}
        </div>
      )}
    </GlassCard>
  );
}
