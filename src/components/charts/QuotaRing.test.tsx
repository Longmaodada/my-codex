import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { QuotaRing } from "./QuotaRing";

describe("QuotaRing", () => {
  it("labels remaining quota explicitly", () => {
    render(<QuotaRing value={73} />);
    expect(screen.getByText("剩余")).toBeInTheDocument();
    expect(screen.getByText("7 天额度")).toBeInTheDocument();
  });

  it("does not show fake values for unavailable quota", () => {
    render(<QuotaRing value={null} />);
    expect(screen.getByText("—")).toBeInTheDocument();
    expect(screen.getByText("数据不可用")).toBeInTheDocument();
  });
});
