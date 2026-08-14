export type MetricSource = "official" | "local" | "estimated" | "mock";
export type DataStatus =
  | "loading"
  | "ok"
  | "stale"
  | "signed_out"
  | "unavailable"
  | "schema_changed";

export interface SourcedValue<T> {
  value: T | null;
  source: MetricSource;
  confidence?: number;
}

export interface TokenBreakdown {
  input: number;
  output: number;
  cached: number;
  reasoning: number;
  total: number;
}

export interface QuotaWindow {
  id: "primary" | "secondary";
  label: string;
  remainingPercent: number | null;
  usedPercent: number | null;
  resetAt: string | null;
  windowMinutes: number | null;
}

export interface QuotaSnapshot {
  status: DataStatus;
  plan: string | null;
  provider: "codex-app-server" | "legacy" | "mock" | "unavailable";
  source: MetricSource;
  sampledAt: string;
  lastSuccessAt: string | null;
  windows: QuotaWindow[];
  message?: string;
}

export interface DailyUsage extends TokenBreakdown {
  date: string;
  requests: number;
  sessions: number;
  activeSeconds: number;
  cacheHitRatio: number;
  source?: MetricSource;
}

export interface ProjectUsage extends TokenBreakdown {
  id: string;
  name: string;
  path: string;
  sessions: number;
  requests: number;
  activeSeconds: number;
  lastActiveAt: string;
  source: MetricSource;
}

export type UsageRange = "today" | "sevenDays" | "thirtyDays" | "ninetyDays" | "all";

export interface ProjectModelUsage extends TokenBreakdown {
  name: string;
  requests: number;
  sessions: number;
  share: number;
  averageTokens: number;
  cacheHitRatio: number;
  lastUsedAt: string;
  source: MetricSource;
}

export interface ProjectSkillUsage extends TokenBreakdown {
  name: string;
  invocations: number;
  share: number;
  averageTokens: number;
  lastUsedAt: string;
  source: MetricSource;
}

export interface ProjectDetail {
  summary: ProjectUsage;
  trend: DailyUsage[];
  models: ProjectModelUsage[];
  skills: ProjectSkillUsage[];
  averageSessionTokens: number;
}

export interface SkillUsage extends TokenBreakdown {
  name: string;
  invocations: number;
  projects: string[];
  lastUsedAt: string;
  /** Cache hit ratio for this skill's attributed input, when available. */
  cacheHitRatio: number | null;
  /** Attribution confidence is unavailable when the backend has no evidence score. */
  confidence: number | null;
  source: MetricSource;
}

export interface ModelUsage extends TokenBreakdown {
  name: string;
  requests: number;
  projects: number;
  cacheHitRatio: number;
  source: MetricSource;
}

export interface TaskSkillUsage {
  name: string;
  invocations: number;
}

export interface TaskUsage extends TokenBreakdown {
  id: string;
  startedAt: string | null;
  endedAt: string | null;
  projectName: string;
  model: string;
  requests: number;
  activeSeconds: number;
  skills: TaskSkillUsage[];
  source: MetricSource;
}

export interface UsageSummary {
  today: TokenBreakdown;
  week: TokenBreakdown;
  last7Days: TokenBreakdown;
  lifetime: TokenBreakdown;
  requestsToday: number;
  sessionsToday: number;
  tasksToday: number;
  cacheHitRatio: number;
  daily: DailyUsage[];
  projects: ProjectUsage[];
  tasks: TaskUsage[];
  skills: SkillUsage[];
  models: ModelUsage[];
  /** Optional user-defined local goals; never interpreted as official quota. */
  goals?: {
    today: number;
    week: number;
  };
}

export type ThemeMode = "light" | "dark" | "system";
export type RefreshMode = "smart" | "10s" | "30s" | "1m" | "5m" | "custom";

export interface AppSettings {
  mockMode: boolean;
  launchAtStartup: boolean;
  showWidgetOnLaunch: boolean;
  closeToTray: boolean;
  alwaysOnTop: boolean;
  capsuleMode: boolean;
  lockPosition: boolean;
  autoCollapse: boolean;
  showInTaskbar: boolean;
  theme: ThemeMode;
  language: "zh-CN" | "en";
  refreshMode: RefreshMode;
  customRefreshSeconds: number;
  notificationThresholds: number[];
  primaryQuotaWindow: "primary" | "secondary";
}

export interface AppSnapshot {
  quota: QuotaSnapshot;
  usage: UsageSummary;
  settings: AppSettings;
}
