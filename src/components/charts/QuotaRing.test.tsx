import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { QuotaRing, quotaColor } from "./QuotaRing";

describe("QuotaRing", () => {
  it("labels remaining quota explicitly", () => {
    render(<QuotaRing value={73} />);
    expect(screen.getByText("剩余")).toBeInTheDocument();
    expect(screen.getByText("7 天额度")).toBeInTheDocument();
  });

  it("uses one color for the whole arc at each threshold", () => {
    expect(quotaColor(90)).toBe("#3267ff");
    expect(quotaColor(60)).toBe("#28b979");
    expect(quotaColor(40)).toBe("#f39a28");
    expect(quotaColor(10)).toBe("#ef4b45");
  });

  it("does not show fake values for unavailable quota", () => {
    render(<QuotaRing value={null} />);
    expect(screen.getByText("—")).toBeInTheDocument();
    expect(screen.getByText("数据不可用")).toBeInTheDocument();
  });
});
