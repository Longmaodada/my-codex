import { invoke } from "@tauri-apps/api/core";
import { defaultSettings } from "../data/defaultSettings";
import { mockSnapshot } from "../data/mockData";
import type { AppSettings, AppSnapshot, TokenBreakdown, UsageSummary } from "../types/analytics";

export const isTauri = () => typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);

const zeroTokens = (): TokenBreakdown => ({
  input: 0,
  output: 0,
  cached: 0,
  reasoning: 0,
  total: 0,
});

const emptyUsage = (): UsageSummary => ({
  today: zeroTokens(),
  week: zeroTokens(),
  last7Days: zeroTokens(),
  lifetime: zeroTokens(),
  requestsToday: 0,
  sessionsToday: 0,
  tasksToday: 0,
  cacheHitRatio: 0,
  daily: [],
  projects: [],
  tasks: [],
  skills: [],
  models: [],
});

function backendErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message) return error.message;
  if (typeof error === "string" && error) return error;
  if (error && typeof error === "object" && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string" && message) return message;
  }
  return "真实数据暂不可用";
}

export function createUnavailableSnapshot(
  error: unknown,
  settings: AppSettings = defaultSettings,
): AppSnapshot {
  const sampledAt = new Date().toISOString();
  return {
    quota: {
      status: "unavailable",
      plan: null,
      provider: "unavailable",
      source: "official",
      sampledAt,
      lastSuccessAt: null,
      message: backendErrorMessage(error),
      windows: [
        { id: "primary", label: "主要额度", remainingPercent: null, usedPercent: null, resetAt: null, windowMinutes: null },
        { id: "secondary", label: "次要额度", remainingPercent: null, usedPercent: null, resetAt: null, windowMinutes: null },
      ],
    },
    usage: emptyUsage(),
    // A failed bootstrap must never imply that mock data was loaded.
    settings: { ...structuredClone(settings), mockMode: false },
  };
}

let browserPreviewSettings: AppSettings = {
  ...structuredClone(mockSnapshot.settings),
  mockMode: true,
};

function browserPreviewSnapshot(): AppSnapshot {
  const snapshot = structuredClone(mockSnapshot);
  snapshot.settings = { ...structuredClone(browserPreviewSettings), mockMode: true };
  return snapshot;
}

export async function loadSnapshot(): Promise<AppSnapshot> {
  if (!isTauri()) return browserPreviewSnapshot();

  try {
    return await invoke<AppSnapshot>("bootstrap");
  } catch (error) {
    let settings = defaultSettings;
    try {
      settings = await invoke<AppSettings>("get_settings");
    } catch {
      // The independent settings read is best-effort. Safe defaults preserve
      // rendering without introducing any usage samples.
    }
    return createUnavailableSnapshot(error, settings);
  }
}

export async function refreshSnapshot(): Promise<AppSnapshot> {
  if (!isTauri()) {
    await new Promise((resolve) => setTimeout(resolve, 650));
    return browserPreviewSnapshot();
  }
  await invoke("refresh_quota_snapshot");
  return invoke<AppSnapshot>("bootstrap");
}

export async function saveSettings(settings: AppSettings): Promise<AppSettings> {
  if (!isTauri()) {
    browserPreviewSettings = { ...structuredClone(settings), mockMode: true };
    return structuredClone(browserPreviewSettings);
  }
  return invoke<AppSettings>("save_settings", { settings });
}

export async function windowAction(
  action: "show_dashboard" | "show_widget" | "open_settings" | "minimize" | "maximize" | "close" | "quit",
) {
  if (!isTauri()) return;
  const commands: Record<typeof action, [string, Record<string, unknown>]> = {
    show_dashboard: ["show_window", { target: "dashboard" }],
    show_widget: ["show_window", { target: "floating" }],
    open_settings: ["show_window", { target: "dashboard", route: "settings" }],
    minimize: ["hide_window", { target: "dashboard" }],
    maximize: ["toggle_window", { target: "dashboard" }],
    close: ["hide_window", { target: "dashboard" }],
    quit: ["exit_application", {}],
  };
  const [command, args] = commands[action];
  await invoke(command, args);
}

export async function setWidgetCompact(compact: boolean) {
  if (!isTauri()) return;
  await invoke("set_floating_compact", { compact });
}

export function __resetBrowserPreviewForTests() {
  browserPreviewSettings = { ...structuredClone(mockSnapshot.settings), mockMode: true };
}
