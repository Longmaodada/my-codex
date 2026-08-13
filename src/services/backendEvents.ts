import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauri } from "./backend";

export type NavigationTarget =
  | "overview"
  | "month"
  | "week"
  | "projects"
  | "skills"
  | "models"
  | "cache"
  | "settings";

const routeAliases: Record<string, NavigationTarget> = {
  overview: "overview",
  home: "overview",
  dashboard: "overview",
  today: "overview",
  month: "month",
  week: "week",
  project: "projects",
  projects: "projects",
  skill: "skills",
  skills: "skills",
  model: "models",
  models: "models",
  cache: "cache",
  settings: "settings",
};

export function normalizeNavigationTarget(payload: unknown): NavigationTarget | null {
  const candidate = typeof payload === "string"
    ? payload
    : payload && typeof payload === "object" && "route" in payload
      ? (payload as { route?: unknown }).route
      : null;
  if (typeof candidate !== "string") return null;
  const normalized = candidate.trim().toLowerCase().replace(/^\/+/, "");
  return routeAliases[normalized] ?? null;
}

interface BackendEventHandlers {
  reload: () => void | Promise<void>;
  navigate: (target: NavigationTarget) => void;
}

export async function subscribeBackendEvents(
  handlers: BackendEventHandlers,
): Promise<UnlistenFn> {
  if (!isTauri()) return () => undefined;

  const unlisteners = await Promise.all([
    listen("my-codex://quota-updated", () => { void handlers.reload(); }),
    listen("my-codex://usage-updated", () => { void handlers.reload(); }),
    listen("my-codex://settings-updated", () => { void handlers.reload(); }),
    listen<unknown>("my-codex://navigate", ({ payload }) => {
      const target = normalizeNavigationTarget(payload);
      if (target) handlers.navigate(target);
    }),
    listen("my-codex://open-settings", () => handlers.navigate("settings")),
  ]);

  return () => unlisteners.forEach((unlisten) => unlisten());
}
