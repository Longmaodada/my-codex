import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SkillsPage } from "./SkillsPage";
import type { SkillUsage } from "../../types/analytics";

const tokens = (total: number): SkillUsage => ({
  name: "",
  input: total,
  output: 0,
  cached: 0,
  reasoning: 0,
  total,
  invocations: 1,
  projects: [],
  lastUsedAt: "今天",
  cacheHitRatio: null,
  confidence: null,
  source: "local",
});

describe("SkillsPage", () => {
  it("keeps cache hit rate separate from attribution confidence and marks missing skill tokens", () => {
    render(
      <SkillsPage
        skills={[
          { ...tokens(100), name: "skill-with-cache", cached: 60, cacheHitRatio: 60, confidence: null },
          { ...tokens(0), name: "skill-without-token" },
        ]}
      />,
    );

    const rowWithCache = screen.getByText("skill-with-cache").closest(".analytics-row");
    const rowWithoutToken = screen.getByText("skill-without-token").closest(".analytics-row");
    expect(rowWithCache).toHaveTextContent("缓存命中60%");
    expect(rowWithCache).toHaveTextContent("归属置信度—");
    expect(rowWithoutToken).toHaveTextContent("Token—");
    expect(rowWithoutToken).toHaveTextContent("缓存命中—");
  });
});
