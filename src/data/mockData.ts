import type { AppSnapshot, DailyUsage, ModelUsage, OfficialResetVoucher, ProjectUsage, SkillUsage, TaskUsage } from "../types/analytics";
import { defaultSettings } from "./defaultSettings";

const now = new Date();
const iso = now.toISOString();
const resetAt = new Date(now.getTime() + 2 * 86_400_000 + 13 * 3_600_000 + 42 * 60_000).toISOString();

/** Demonstration-only entries; real mode stays unavailable until an official source is connected. */
export const officialResetVouchers: OfficialResetVoucher[] = [
  { id: "voucher-01", expiresAt: "2026-08-24T23:59:59+08:00", status: "available", source: "mock", label: "8 月额度卷 1" },
  { id: "voucher-02", expiresAt: "2026-08-31T23:59:59+08:00", status: "available", source: "mock" },
  { id: "voucher-03", expiresAt: "2026-09-07T23:59:59+08:00", status: "available", source: "mock" },
];

const split = (total: number) => ({
  input: Math.round(total * 0.51),
  output: Math.round(total * 0.21),
  cached: Math.round(total * 0.23),
  reasoning: Math.round(total * 0.05),
  total,
});

const splitSkill = (total: number, cacheHitRatio: number) => {
  const tokens = split(total);
  return {
    ...tokens,
    cached: Math.round(tokens.input * cacheHitRatio / 100),
  };
};

function dateKey(offset: number) {
  const d = new Date(now);
  d.setDate(d.getDate() - offset);
  return d.toISOString().slice(0, 10);
}

const MOCK_TODAY = 2_940_000_000;
const MOCK_LAST_7_DAYS = 6_800_000_000;
const MOCK_LIFETIME = 8_600_000_000;
const recentSevenTokens = [520_000_000, 590_000_000, 630_000_000, 670_000_000, 710_000_000, 740_000_000, MOCK_TODAY];

export const dailyUsage: DailyUsage[] = Array.from({ length: 90 }, (_, index) => {
  const offset = 89 - index;
  const wave = (Math.sin(index * 0.62) + 1) / 2;
  const recentIndex = index - 83;
  const total = recentIndex >= 0
    ? recentSevenTokens[recentIndex]
    : Math.round((9_000_000 + wave * 20_000_000) * (index % 7 === 0 ? 0.35 : 1));
  return {
    date: dateKey(offset),
    ...split(total),
    requests: Math.round(12 + wave * 172),
    sessions: Math.round(1 + wave * 20),
    activeSeconds: Math.round(720 + wave * 13_800),
    cacheHitRatio: Math.round(42 + wave * 43),
    source: "mock",
  };
});

export const projects: ProjectUsage[] = [
  ["project-config", "Code项目配置生成工具", "D:\\Projects\\code-project-config", 2_580_000_000, 49, 184, 20_484, "3分钟前"],
  ["pet-skills-2", "hatch-pet-users-zhitong-codex-skills-2", "D:\\Projects\\hatch-pet-users-zhitong-codex-skills-2", 2_400_000_000, 53, 167, 15_840, "18分钟前"],
  ["pet-skills", "hatch-pet-users-zhitong-codex-skills", "D:\\Projects\\hatch-pet-users-zhitong-codex-skills", 563_200_000, 34, 96, 11_460, "1小时前"],
  ["weather", "天气桌面组件分析工具", "D:\\Projects\\weather", 478_100_000, 24, 72, 10_980, "2小时前"],
  ["proxy", "内网代理池维护", "D:\\Projects\\proxy-pool", 335_900_000, 14, 61, 5_640, "昨天"],
  ["asr", "ASR桌面系统", "D:\\Projects\\asr-desktop", 223_400_000, 8, 38, 3_240, "2天前"],
].map(([id, name, path, total, sessions, requests, activeSeconds, lastActiveAt]) => ({
  id: id as string,
  name: name as string,
  path: path as string,
  ...split(total as number),
  sessions: sessions as number,
  requests: requests as number,
  activeSeconds: activeSeconds as number,
  lastActiveAt: lastActiveAt as string,
  source: "mock",
}));

export const skills: SkillUsage[] = [
  ["frontend-design", 42, 1_840_000_000, ["Code项目配置生成工具", "ASR桌面系统"], 0.88, 78],
  ["github", 31, 1_220_000_000, ["hatch-pet-users-zhitong-codex-skills-2"], 0.96, 64],
  ["browser", 24, 860_000_000, ["Code项目配置生成工具"], 0.92, 51],
  ["security-analysis", 18, 620_000_000, ["内网代理池维护"], 0.78, 39],
  ["pdf", 12, 380_000_000, ["ASR桌面系统"], 0.81, 27],
].map(([name, invocations, total, linkedProjects, confidence, cacheHitRatio]) => ({
  name: name as string,
  invocations: invocations as number,
  ...splitSkill(total as number, cacheHitRatio as number),
  projects: linkedProjects as string[],
  lastUsedAt: "今天",
  cacheHitRatio: (total as number) > 0 ? cacheHitRatio as number : null,
  confidence: confidence as number,
  source: "mock",
}));

export const models: ModelUsage[] = [
  ["GPT-5.6", 3_180_000_000, 86, 5, 82],
  ["GPT-5.6 Sol", 2_720_000_000, 54, 3, 80],
  ["GPT-5.5", 1_410_000_000, 31, 4, 74],
  ["GPT-5.4 mini", 890_000_000, 42, 2, 85],
  ["其他", 400_000_000, 18, 3, 61],
].map(([name, total, requests, modelProjects, cacheHitRatio]) => ({
  name: name as string,
  ...split(total as number),
  requests: requests as number,
  projects: modelProjects as number,
  cacheHitRatio: cacheHitRatio as number,
  source: "mock",
})) as ModelUsage[];

export const tasks: TaskUsage[] = [
  ["task-001", "Code项目配置生成工具", "GPT-5.6", 420_000_000, ["frontend-design", "browser"]],
  ["task-002", "hatch-pet-users-zhitong-codex-skills-2", "GPT-5.6", 360_000_000, ["github"]],
  ["task-003", "Code项目配置生成工具", "GPT-5.6 Sol", 310_000_000, ["frontend-design", "impeccable"]],
  ["task-004", "内网代理池维护", "GPT-5.5", 260_000_000, ["security-analysis"]],
  ["task-005", "ASR桌面系统", "GPT-5.6", 220_000_000, ["pdf"]],
  ["task-006", "Code项目配置生成工具", "GPT-5.6", 190_000_000, ["browser", "visualize"]],
].map(([id, projectName, model, total, skillNames], index) => ({
  id: id as string,
  startedAt: new Date(now.getTime() - index * 19 * 60_000).toISOString(),
  endedAt: new Date(now.getTime() - index * 19 * 60_000 + 18 * 60_000).toISOString(),
  projectName: projectName as string,
  model: model as string,
  ...split(total as number),
  requests: 1,
  activeSeconds: 18 * 60,
  skills: (skillNames as string[]).map((name) => ({ name, invocations: 1 })),
  source: "mock",
}));

export const mockSnapshot: AppSnapshot = {
  quota: {
    status: "ok",
    plan: "Codex Pro",
    provider: "mock",
    source: "mock",
    sampledAt: iso,
    lastSuccessAt: iso,
    windows: [
      { id: "primary", label: "5 小时额度", remainingPercent: 86, usedPercent: 14, resetAt: new Date(now.getTime() + 2.3 * 3_600_000).toISOString(), windowMinutes: 300 },
      { id: "secondary", label: "7 天额度", remainingPercent: 73, usedPercent: 27, resetAt, windowMinutes: 10_080 },
    ],
    message: "展示数据，仅用于验证完整界面",
  },
  officialResetVouchers,
  usage: {
    today: split(MOCK_TODAY),
    week: split(MOCK_TODAY),
    last7Days: split(MOCK_LAST_7_DAYS),
    lifetime: split(MOCK_LIFETIME),
    requestsToday: 184,
    sessionsToday: 22,
    tasksToday: tasks.length,
    cacheHitRatio: 80,
    daily: dailyUsage,
    projects,
    tasks,
    skills,
    models,
    goals: { today: 3_000_000_000, week: 20_000_000_000 },
  },
  settings: { ...defaultSettings, mockMode: true },
};
