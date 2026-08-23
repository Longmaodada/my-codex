import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ResetVoucherCard, summarizeResetVouchers, type ResetVoucher } from "./ResetVoucherCard";

const vouchers: ResetVoucher[] = [
  { id: "voucher-1", expiresAt: "2026-09-30", status: "available", label: "9 月额度卷" },
  { id: "voucher-2", expiresAt: "2026-10-15", status: "expired" },
];

describe("ResetVoucherCard", () => {
  it("明确展示官方重置卷总数、到期日期和状态", () => {
    render(<ResetVoucherCard vouchers={vouchers} />);

    expect(screen.getByRole("heading", { name: "Codex 额度重置卷" })).toBeInTheDocument();
    expect(screen.getByText("官方重置卷")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
    expect(screen.getByText("当前可使用")).toBeInTheDocument();
    expect(screen.getByText("9 月额度卷")).toBeInTheDocument();
    expect(screen.getByText("官方重置卷 2")).toBeInTheDocument();
    expect(screen.getByText("到期：2026年9月30日")).toBeInTheDocument();
    expect(screen.getByText("到期：2026年10月15日")).toBeInTheDocument();
    expect(screen.getByText("可使用")).toBeInTheDocument();
    expect(screen.getByText("已过期")).toBeInTheDocument();
  });

  it("强调卷不会重置软件数据", () => {
    render(<ResetVoucherCard vouchers={vouchers} />);

    expect(screen.getByText(/这不是软件数据重置/)).toBeInTheDocument();
    expect(screen.getByText(/不会删除 My Codex 的本地统计/)).toBeInTheDocument();
  });

  it("支持没有卷的空状态", () => {
    render(<ResetVoucherCard vouchers={[]} />);

    expect(screen.getByText("持有总数")).toBeInTheDocument();
    expect(screen.getByText("0")).toBeInTheDocument();
    expect(screen.getByText("当前没有官方重置卷")).toBeInTheDocument();
    expect(screen.getByText("当官方重置卷到账后，会在这里显示每张卷的到期日期。")).toBeInTheDocument();
  });

  it("支持官方卷信息不可用状态并隐藏可能过期的明细", () => {
    render(<ResetVoucherCard unavailable vouchers={vouchers} unavailableMessage="官方账户信息暂时不可读。" />);

    expect(screen.getByRole("alert")).toBeInTheDocument();
    expect(screen.getByText("获取失败：官方重置卷暂不可用")).toBeInTheDocument();
    expect(screen.getByText("官方账户信息暂时不可读。")).toBeInTheDocument();
    expect(screen.getByText("下一步：稍后刷新。")).toBeInTheDocument();
    expect(screen.getByText("持有总数")).toBeInTheDocument();
    expect(screen.getAllByText("—")).toHaveLength(2);
    expect(screen.queryByText("9 月额度卷")).not.toBeInTheDocument();
  });

  it("把不可用和已成功获取但为空区分开", () => {
    expect(summarizeResetVouchers(undefined)).toEqual({ total: null, available: null, unavailable: true });
    expect(summarizeResetVouchers([])).toEqual({ total: 0, available: 0, unavailable: false });
    expect(summarizeResetVouchers(vouchers)).toEqual({ total: 2, available: 1, unavailable: false });
  });
});
