import type { ThemeMode } from "../types/analytics";

export type ResolvedTheme = Exclude<ThemeMode, "system">;

export function resolveTheme(theme: ThemeMode, systemIsDark?: boolean): ResolvedTheme {
  if (theme !== "system") return theme;
  const isDark = systemIsDark ?? window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;
  return isDark ? "dark" : "light";
}

export function installTheme(theme: ThemeMode): () => void {
  const media = window.matchMedia?.("(prefers-color-scheme: dark)");
  const apply = () => {
    document.documentElement.dataset.theme = resolveTheme(theme, media?.matches);
  };
  apply();
  if (theme !== "system" || !media) return () => undefined;
  media.addEventListener?.("change", apply);
  return () => media.removeEventListener?.("change", apply);
}
