import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { DataResetCard } from "./DataResetCard";

describe("DataResetCard", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("confirms and allows the reset action to be used repeatedly", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    const onReset = vi.fn().mockResolvedValue(undefined);
    render(<DataResetCard onReset={onReset} />);

    const resetButton = screen.getByRole("button", { name: "重置数据" });
    fireEvent.click(resetButton);
    await waitFor(() => expect(onReset).toHaveBeenCalledTimes(1));
    expect(screen.getByText("已重置，本地统计已刷新")).toBeInTheDocument();

    fireEvent.click(resetButton);
    await waitFor(() => expect(onReset).toHaveBeenCalledTimes(2));
  });
});
