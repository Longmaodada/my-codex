import { beforeEach, describe, expect, it } from "vitest";
import { defaultSettings } from "../data/defaultSettings";
import { mockSnapshot } from "../data/mockData";
import type { AppSnapshot } from "../types/analytics";
import { createUnavailableSnapshot, __resetBrowserPreviewForTests, loadSnapshot } from "./backend";

describe("bootstrap snapshot contract", () => {
  beforeEach(() => {
    delete window.__TAURI_INTERNALS__;
    __resetBrowserPreviewForTests();
  });

  it("accepts the real bootstrap shape without requiring a frontend-only source field", () => {
    const bootstrapSnapshot = {
      ...mockSnapshot,
      officialResetVouchers: [
        { id: "credit-1", expiresAt: "2026-09-30T23:59:59Z", status: "available" as const },
      ],
    } satisfies AppSnapshot;

    expect(bootstrapSnapshot.officialResetVouchers).toEqual([
      { id: "credit-1", expiresAt: "2026-09-30T23:59:59Z", status: "available" },
    ]);
  });

  it("keeps the unavailable snapshot official field explicit and never substitutes mock vouchers", () => {
    const snapshot = createUnavailableSnapshot(new Error("account/rateLimits/read failed"), defaultSettings);

    expect(snapshot.officialResetVouchers).toBeNull();
    expect(snapshot.settings.mockMode).toBe(false);
    expect(snapshot.usage.projects).toEqual([]);
  });

  it("returns a cloned browser mock with mock vouchers and does not leak mutations", async () => {
    const snapshot = await loadSnapshot();

    expect(snapshot.settings.mockMode).toBe(true);
    expect(snapshot.officialResetVouchers).toEqual(mockSnapshot.officialResetVouchers);
    expect(snapshot.officialResetVouchers?.every((voucher) => voucher.source === "mock")).toBe(true);

    snapshot.officialResetVouchers?.pop();
    const nextSnapshot = await loadSnapshot();
    expect(nextSnapshot.officialResetVouchers).toHaveLength(mockSnapshot.officialResetVouchers?.length ?? 0);
  });
});
