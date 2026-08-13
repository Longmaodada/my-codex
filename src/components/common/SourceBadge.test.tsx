import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SourceBadge } from "./SourceBadge";

describe("SourceBadge", () => {
  it("makes local estimates visible", () => {
    render(<SourceBadge source="estimated" />);
    expect(screen.getByText("本地估算")).toHaveAttribute("title", expect.stringContaining("并非官方"));
  });
});
