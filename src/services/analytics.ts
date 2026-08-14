import { invoke } from "@tauri-apps/api/core";
import { dailyUsage, models as mockModels, projects as mockProjects, skills as mockSkills } from "../data/mockData";
import type {
  DailyUsage,
  MetricSource,
  ProjectDetail,
  ProjectModelUsage,
  ProjectSkillUsage,
  ProjectUsage,
  TokenBreakdown,
  UsageRange,
} from "../types/analytics";

interface RawTokens {
  inputTokens: number;
  outputTokens: number;
  cachedInputTokens: number;
  reasoningTokens: number;
  totalTokens: number;
}

interface RawProject extends RawTokens {
  projectId: string;
  projectName: string;
  projectPath: string | null;
  requests: number;
  sessions: number;
  activeSeconds: number;
  lastUsedAt: string | null;
  source: MetricSource;
}

interface RawTrendPoint extends RawTokens {
  bucket: string;
  requests: number;
  sessions: number;
  activeSeconds: number;
  cacheHitRatio: number;
  source: MetricSource;
}

interface RawModel extends RawTokens {
  model: string;
  requests: number;
  sessions: number;
  share: number;
  averageTokens: number;
  cacheHitRatio: number;
  lastUsedAt: string | null;
  source: MetricSource;
}

interface RawSkill extends RawTokens {
  skillName: string;
  invocations: number;
  share: number;
  averageTokens: number;
  lastUsedAt: string | null;
  source: MetricSource;
}

interface RawTaskSkill {
  name: string;
  invocations: number;
}

interface RawTask extends RawTokens {
  id: string;
  startedAt: string | null;
  endedAt: string | null;
  projectName: string;
  model: string;
  requests: number;
  activeSeconds: number;
  skills: RawTaskSkill[];
  source: MetricSource;
}

interface RawProjectDetail {
  summary: RawProject;
  trend: RawTrendPoint[];
  models: RawModel[];
  skills: RawSkill[];
  averageSessionTokens: number;
}

const RANGE_SCALE: Record<UsageRange, number> = {
  today: 0.14,
  sevenDays: 0.46,
  thirtyDays: 0.78,
  ninetyDays: 0.92,
  all: 1,
};

const RANGE_DAYS: Record<UsageRange, number> = {
  today: 1,
  sevenDays: 7,
  thirtyDays: 30,
  ninetyDays: 90,
  all: 90,
};

const isTauri = () => typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);

function mapTokens(raw: RawTokens): TokenBreakdown {
  return {
    input: raw.inputTokens,
    output: raw.outputTokens,
    cached: raw.cachedInputTokens,
    reasoning: raw.reasoningTokens,
    total: raw.totalTokens,
  };
}

export function mapProject(raw: RawProject): ProjectUsage {
  return {
    id: raw.projectId,
    name: raw.projectName,
    path: raw.projectPath ?? "",
    ...mapTokens(raw),
    sessions: raw.sessions,
    requests: raw.requests,
    activeSeconds: raw.activeSeconds,
    lastActiveAt: raw.lastUsedAt ?? "",
    source: raw.source,
  };
}

function mapTrend(raw: RawTrendPoint): DailyUsage {
  return {
    date: raw.bucket,
    ...mapTokens(raw),
    requests: raw.requests,
    sessions: raw.sessions,
    activeSeconds: raw.activeSeconds,
    cacheHitRatio: raw.cacheHitRatio,
    source: raw.source,
  };
}

function mapModel(raw: RawModel): ProjectModelUsage {
  return {
    name: raw.model,
    ...mapTokens(raw),
    requests: raw.requests,
    sessions: raw.sessions,
    share: raw.share,
    averageTokens: raw.averageTokens,
    cacheHitRatio: raw.cacheHitRatio,
    lastUsedAt: raw.lastUsedAt ?? "",
    source: raw.source,
  };
}

function mapSkill(raw: RawSkill): ProjectSkillUsage {
  return {
    name: raw.skillName,
    ...mapTokens(raw),
    invocations: raw.invocations,
    share: raw.share,
    averageTokens: raw.averageTokens,
    lastUsedAt: raw.lastUsedAt ?? "",
    source: raw.source,
  };
}

function scale(value: number, factor: number) {
  return Math.max(0, Math.round(value * factor));
}

function scaleProject(project: ProjectUsage, factor: number): ProjectUsage {
  return {
    ...project,
    input: scale(project.input, factor),
    output: scale(project.output, factor),
    cached: scale(project.cached, factor),
    reasoning: scale(project.reasoning, factor),
    total: scale(project.total, factor),
    sessions: scale(project.sessions, factor),
    requests: scale(project.requests, factor),
    activeSeconds: scale(project.activeSeconds, factor),
    source: "mock",
  };
}

/** Browser/Tauri mock-mode data is intentionally range-derived and remains visibly mock. */
export function deriveMockProjects(base: ProjectUsage[], range: UsageRange, limit = 50) {
  const maximumItems = range === "today" ? 2 : range === "sevenDays" ? 4 : base.length;
  return base
    .slice(0, maximumItems)
    .map((project, index) => scaleProject(project, RANGE_SCALE[range] * (1 - index * 0.025)))
    .filter((project) => project.total > 0)
    .sort((left, right) => right.total - left.total)
    .slice(0, limit);
}

function allocate(total: number, weights: number[]) {
  const weightTotal = weights.reduce((sum, value) => sum + value, 0) || 1;
  let used = 0;
  return weights.map((weight, index) => {
    const value = index === weights.length - 1 ? total - used : Math.round(total * weight / weightTotal);
    used += value;
    return Math.max(0, value);
  });
}

function projectSeed(project: ProjectUsage) {
  return [...project.id].reduce((sum, character) => sum + character.charCodeAt(0), 0);
}

function mockLinkedModels(summary: ProjectUsage): ProjectModelUsage[] {
  const shares = [0.62, 0.38];
  return mockModels.slice(0, 2).map((model, index) => {
    const factor = shares[index];
    const total = scale(summary.total, factor);
    const requests = Math.max(0, scale(summary.requests, factor));
    return {
      name: model.name,
      input: scale(summary.input, factor),
      output: scale(summary.output, factor),
      cached: scale(summary.cached, factor),
      reasoning: scale(summary.reasoning, factor),
      total,
      requests,
      sessions: scale(summary.sessions, factor),
      share: factor * 100,
      averageTokens: requests > 0 ? Math.round(total / requests) : 0,
      cacheHitRatio: summary.input > 0 ? Math.round(summary.cached / summary.input * 100) : 0,
      lastUsedAt: summary.lastActiveAt,
      source: "mock",
    };
  });
}

function mockLinkedSkills(summary: ProjectUsage): ProjectSkillUsage[] {
  const shares = [0.56, 0.44];
  return mockSkills.slice(0, 2).map((skill, index) => {
    const factor = shares[index];
    const total = scale(summary.total, factor);
    const invocations = Math.max(1, scale(skill.invocations, RANGE_SCALE.thirtyDays * factor));
    return {
      name: skill.name,
      input: scale(summary.input, factor),
      output: scale(summary.output, factor),
      cached: scale(summary.cached, factor),
      reasoning: scale(summary.reasoning, factor),
      total,
      invocations,
      share: factor * 100,
      averageTokens: Math.round(total / invocations),
      lastUsedAt: summary.lastActiveAt,
      source: "mock",
    };
  });
}

export function deriveMockProjectDetail(project: ProjectUsage, range: UsageRange): ProjectDetail {
  const canonical = mockProjects.find((candidate) =>
    candidate.id === project.id || candidate.path === project.path || candidate.name === project.name,
  ) ?? project;
  const summary = deriveMockProjects([canonical], range, 1)[0] ?? scaleProject(canonical, RANGE_SCALE[range]);
  const days = RANGE_DAYS[range];
  const dates = dailyUsage.slice(-days).map((day) => day.date);
  const seed = projectSeed(summary);
  const weights = dates.map((_, index) => 0.55 + ((Math.sin((index + seed) * 0.73) + 1) / 2));
  const totals = allocate(summary.total, weights);
  const inputs = allocate(summary.input, weights);
  const outputs = allocate(summary.output, weights);
  const cached = allocate(summary.cached, weights);
  const reasoning = allocate(summary.reasoning, weights);
  const requests = allocate(summary.requests, weights);
  const sessions = allocate(summary.sessions, weights);
  const activeSeconds = allocate(summary.activeSeconds, weights);
  const trend = dates.map((date, index): DailyUsage => ({
    date,
    input: inputs[index],
    output: outputs[index],
    cached: cached[index],
    reasoning: reasoning[index],
    total: totals[index],
    requests: requests[index],
    sessions: sessions[index],
    activeSeconds: activeSeconds[index],
    cacheHitRatio: inputs[index] > 0 ? Math.round(cached[index] / inputs[index] * 100) : 0,
    source: "mock",
  }));
  return {
    summary,
    trend,
    models: mockLinkedModels(summary),
    skills: mockLinkedSkills(summary),
    averageSessionTokens: summary.sessions > 0 ? Math.round(summary.total / summary.sessions) : 0,
  };
}

export async function getProjects(
  range: UsageRange,
  limit = 50,
  seedProjects?: ProjectUsage[],
): Promise<ProjectUsage[]> {
  const mockBase = seedProjects?.length ? seedProjects : mockProjects;
  if (!isTauri() || mockBase.every((project) => project.source === "mock")) {
    return deriveMockProjects(mockBase, range, limit);
  }
  const response = await invoke<RawProject[]>("get_projects", { query: { range, limit } });
  return response.map(mapProject);
}

export async function getProjectDetail(
  projectId: string,
  range: UsageRange,
  mockProject?: ProjectUsage,
): Promise<ProjectDetail | null> {
  if (!isTauri() || mockProject?.source === "mock") {
    const project = mockProject ?? mockProjects.find((candidate) => candidate.id === projectId);
    return project ? deriveMockProjectDetail(project, range) : null;
  }
  const response = await invoke<RawProjectDetail | null>("get_project_detail", { projectId, range });
  if (!response) return null;
  return {
    summary: mapProject(response.summary),
    trend: response.trend.map(mapTrend),
    models: response.models.map(mapModel),
    skills: response.skills.map(mapSkill),
    averageSessionTokens: response.averageSessionTokens,
  };
}
