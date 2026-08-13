import { create } from "zustand";
import { loadSnapshot, refreshSnapshot, saveSettings } from "../services/backend";
import type { NavigationTarget } from "../services/backendEvents";
import type { AppSettings, AppSnapshot } from "../types/analytics";

interface AppState {
  snapshot: AppSnapshot | null;
  loading: boolean;
  refreshing: boolean;
  error: string | null;
  lastRefreshedAt: Date | null;
  navigationTarget: NavigationTarget | null;
  initialize: () => Promise<void>;
  reload: () => Promise<void>;
  refresh: () => Promise<void>;
  updateSettings: (patch: Partial<AppSettings>) => Promise<void>;
  navigate: (target: NavigationTarget) => void;
}

let refreshFlight: Promise<void> | null = null;
let reloadFlight: Promise<void> | null = null;

export const useAppStore = create<AppState>((set, get) => ({
  snapshot: null,
  loading: true,
  refreshing: false,
  error: null,
  lastRefreshedAt: null,
  navigationTarget: null,
  initialize: async () => {
    try {
      const snapshot = await loadSnapshot();
      set({ snapshot, loading: false, error: null, lastRefreshedAt: new Date() });
    } catch (error) {
      set({ loading: false, error: error instanceof Error ? error.message : "初始化失败" });
    }
  },
  reload: async () => {
    if (reloadFlight) return reloadFlight;
    reloadFlight = (async () => {
      try {
        const snapshot = await loadSnapshot();
        set({ snapshot, loading: false, error: null, lastRefreshedAt: new Date() });
      } catch (error) {
        set({ error: error instanceof Error ? error.message : "数据重载失败" });
      } finally {
        reloadFlight = null;
      }
    })();
    return reloadFlight;
  },
  refresh: async () => {
    if (refreshFlight) return refreshFlight;
    refreshFlight = (async () => {
      set({ refreshing: true });
      try {
        const snapshot = await refreshSnapshot();
        set({ snapshot, error: null, lastRefreshedAt: new Date() });
      } catch (error) {
        set({ error: error instanceof Error ? error.message : "刷新失败" });
      } finally {
        set({ refreshing: false });
        refreshFlight = null;
      }
    })();
    return refreshFlight;
  },
  updateSettings: async (patch) => {
    const current = get().snapshot;
    if (!current) return;
    const next = { ...current.settings, ...patch };
    set({ snapshot: { ...current, settings: next } });
    try {
      await saveSettings(next);
      // Locking only changes native window behavior. Avoid reloading the whole
      // snapshot here: a reload reapplies capsule geometry while the pointer
      // is still over the expanded widget.
      if ("mockMode" in patch) await get().reload();
    } catch (error) {
      set({ error: error instanceof Error ? error.message : "设置保存失败" });
    }
  },
  navigate: (target) => set({ navigationTarget: target }),
}));

export function __resetRefreshFlightForTests() {
  refreshFlight = null;
  reloadFlight = null;
}
