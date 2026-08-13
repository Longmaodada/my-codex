import type { AppSettings } from "../types/analytics";

/** Safe configuration defaults. No usage or quota samples live here. */
export const defaultSettings: AppSettings = {
  mockMode: false,
  launchAtStartup: false,
  showWidgetOnLaunch: true,
  closeToTray: true,
  alwaysOnTop: true,
  capsuleMode: true,
  lockPosition: false,
  autoCollapse: true,
  showInTaskbar: false,
  theme: "system",
  language: "zh-CN",
  refreshMode: "smart",
  customRefreshSeconds: 30,
  notificationThresholds: [50, 25, 20, 10, 5],
  primaryQuotaWindow: "secondary",
};
