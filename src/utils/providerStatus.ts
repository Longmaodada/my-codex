import type { QuotaSnapshot } from "../types/analytics";

export function providerStatusLabel(quota: QuotaSnapshot, mockMode: boolean): string {
  if (mockMode || quota.provider === "mock") return "Mock Mode";
  switch (quota.status) {
    case "loading":
      return "正在连接";
    case "ok":
      return quota.provider === "codex-app-server" ? "已连接" : "数据可用";
    case "stale":
      return "数据已过期";
    case "signed_out":
      return "未登录 Codex";
    case "schema_changed":
      return "接口不兼容";
    case "unavailable":
      return "数据不可用";
  }
}
