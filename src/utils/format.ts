export function formatTokens(value: number | null | undefined, precision = 1) {
  if (value === null || value === undefined || !Number.isFinite(value)) return "—";
  const absolute = Math.abs(value);
  if (absolute >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(precision)}B`;
  if (absolute >= 1_000_000) return `${(value / 1_000_000).toFixed(precision)}M`;
  if (absolute >= 1_000) return `${(value / 1_000).toFixed(precision)}K`;
  return value.toLocaleString("zh-CN");
}

export function formatDuration(seconds: number) {
  if (!Number.isFinite(seconds) || seconds <= 0) return "0 分钟";
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  return hours > 0 ? `${hours}小时 ${minutes}分` : `${minutes}分钟`;
}

export function formatCountdown(resetAt: string | null, now = Date.now()) {
  if (!resetAt) return "重置时间不可用";
  const delta = new Date(resetAt).getTime() - now;
  if (!Number.isFinite(delta) || delta <= 0) return "即将刷新";
  const totalSeconds = Math.floor(delta / 1000);
  const days = Math.floor(totalSeconds / 86_400);
  const hours = Math.floor((totalSeconds % 86_400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  if (days > 0) return `${days}天 ${hours}小时 ${minutes}分`;
  if (hours > 0) return `${hours}小时 ${minutes}分`;
  return `${minutes}分 ${seconds}秒`;
}

export function formatDateTime(value: string | null) {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "—";
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
}

export function percentage(part: number, total: number) {
  if (total <= 0) return 0;
  return Math.max(0, Math.min(100, Math.round((part / total) * 100)));
}
