import { describe, it, expect } from "vitest";
import { formatMemory, VERDICT_THRESHOLDS, verdictForScore } from "./format";

describe("formatMemory", () => {
  it("formats numbers with thousands separators in en-US (no unit)", () => {
    expect(formatMemory(16384, "en")).toBe("16,384");
  });

  it("formats with Bengali digits in bn", () => {
    // Intl bn-BD uses Bengali numerals
    expect(formatMemory(16384, "bn")).toMatch(/১৬,৩৮৪|১৬৩৮৪/);
  });
});

describe("verdict thresholds", () => {
  it("maps scores to the right verdicts", () => {
    expect(verdictForScore(0)).toBe("walk-away");
    expect(verdictForScore(4.9)).toBe("walk-away");
    expect(verdictForScore(5)).toBe("negotiate");
    expect(verdictForScore(6.9)).toBe("negotiate");
    expect(verdictForScore(7)).toBe("good-buy"); // 7+ is a good buy
    expect(verdictForScore(10)).toBe("good-buy");
  });

  it("thresholds are ordered and inside score range", () => {
    expect(VERDICT_THRESHOLDS.goodBuy).toBeGreaterThan(
      VERDICT_THRESHOLDS.negotiate,
    );
    expect(VERDICT_THRESHOLDS.negotiate).toBeGreaterThanOrEqual(0);
    expect(VERDICT_THRESHOLDS.goodBuy).toBeLessThanOrEqual(10);
  });
});
