import { render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { mockSnapshot } from "../../data/mockData";
import { createUnavailableSnapshot } from "../../services/backend";
import type { AppSnapshot } from "../../types/analytics";
import { OverviewPage } from "./OverviewPage";

vi.mock("../../components/charts/UsageTrend", () => ({ UsageTrend: () => null }));
vi.mock("../../services/analytics", () => ({ getProjects: vi.fn(() => new Promise(() => undefined)) }));

const onProject = vi.fn();

function renderOverview(snapshot: AppSnapshot) {
  return render(<OverviewPage snapshot={snapshot} onProject={onProject} />);
}

describe("OverviewPage official reset vouchers", () => {
  it("shows count, expiry and status when the bootstrap contains official vouchers", () => {
    renderOverview({
      ...mockSnapshot,
      officialResetVoucherAvailableCount: 1,
      officialResetVouchers: [
        { id: "credit-1", expiresAt: "2026-09-30T23:59:59Z", status: "available", label: "9 月额度卷" },
        { id: "credit-2", expiresAt: "2026-10-15T23:59:59Z", status: "expired" },
      ],
    });

    const card = screen.getByRole("region", { name: "官方重置卷" });
    expect(within(card).getByText("2")).toBeInTheDocument();
    expect(within(card).getByText("9 月额度卷")).toBeInTheDocument();
    expect(within(card).getByText("到期：2026年9月30日")).toBeInTheDocument();
    expect(within(card).getByText("到期：2026年10月15日")).toBeInTheDocument();
    expect(within(card).getByText("可使用")).toBeInTheDocument();
    expect(within(card).getByText("已过期")).toBeInTheDocument();
  });

  it("uses the official available count when detail rows are incomplete", () => {
    renderOverview({ ...mockSnapshot, officialResetVoucherAvailableCount: 1, officialResetVouchers: [] });

    const card = screen.getByRole("region", { name: "官方重置卷" });
    expect(within(card).getByText("1 张")).toBeInTheDocument();
    expect(within(card).getByText("当前没有官方重置卷")).toBeInTheDocument();
  });

  it("keeps an explicit unavailable empty state when bootstrap omits the field", () => {
    const snapshot = structuredClone(mockSnapshot);
    delete snapshot.officialResetVouchers;
    renderOverview(snapshot);

    const card = screen.getByRole("region", { name: "官方重置卷" });
    expect(within(card).getByRole("alert")).toBeInTheDocument();
    expect(within(card).getByText("获取失败：官方重置卷暂不可用")).toBeInTheDocument();
    expect(within(card).getAllByText("—")).toHaveLength(2);
    expect(within(card).queryByText("当前没有官方重置卷")).not.toBeInTheDocument();
  });

  it("distinguishes a successful empty result from an unavailable field", () => {
    renderOverview({ ...mockSnapshot, officialResetVouchers: [] });

    const card = screen.getByRole("region", { name: "官方重置卷" });
    expect(within(card).getByText("0")).toBeInTheDocument();
    expect(within(card).getByText("当前没有官方重置卷")).toBeInTheDocument();
    expect(within(card).queryByRole("alert")).not.toBeInTheDocument();
  });

  it("does not render the voucher card on the separate monthly trend page", () => {
    render(<OverviewPage snapshot={mockSnapshot} onProject={onProject} defaultRange="30D" />);

    expect(screen.queryByRole("region", { name: "官方重置卷" })).not.toBeInTheDocument();
  });

  it("renders the unavailable snapshot without implying mock data", () => {
    renderOverview(createUnavailableSnapshot("official account unavailable"));

    expect(screen.getByRole("region", { name: "官方重置卷" })).toBeInTheDocument();
    expect(screen.getByText("获取官方重置卷失败，暂时无法获取信息。")).toBeInTheDocument();
    expect(screen.queryByText("8 月额度卷 1")).not.toBeInTheDocument();
  });
});
