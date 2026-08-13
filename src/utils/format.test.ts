import { describe, expect, it } from "vitest";
import { formatCountdown, formatDuration, formatTokens, percentage } from "./format";

describe("format helpers", () => {
  it("formats large token counts without inventing values", () => {
    expect(formatTokens(2_900_000_000)).toBe("2.9B");
    expect(formatTokens(null)).toBe("—");
  });

  it("keeps percentages bounded", () => {
    expect(percentage(150, 100)).toBe(100);
    expect(percentage(1, 0)).toBe(0);
  });

  it("calculates countdown locally", () => {
    const start = Date.UTC(2026, 7, 12, 10, 0, 0);
    expect(formatCountdown(new Date(start + 2 * 86_400_000 + 3 * 3_600_000 + 42 * 60_000).toISOString(), start)).toBe("2天 3小时 42分");
    expect(formatCountdown(new Date(start + 58 * 60_000 + 32_000).toISOString(), start)).toBe("58分 32秒");
  });

  it("formats active duration", () => {
    expect(formatDuration(15_120)).toBe("4小时 12分");
  });
});
