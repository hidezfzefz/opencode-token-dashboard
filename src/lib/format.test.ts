import { describe, expect, it } from "vitest";
import { formatCompact, formatNumber, formatPercent } from "./format";

describe("formatNumber", () => {
  it("groups thousands", () => {
    expect(formatNumber(1234567)).toBe("1,234,567");
  });
});

describe("formatCompact", () => {
  it("uses compact notation for large values", () => {
    expect(formatCompact(1_714_808_757)).toBe("1.7B");
  });

  it("keeps small values exact", () => {
    expect(formatCompact(842)).toBe("842");
  });
});

describe("formatPercent", () => {
  it("returns a rounded share", () => {
    expect(formatPercent(1, 3)).toBe("33%");
  });

  it("handles an empty whole", () => {
    expect(formatPercent(5, 0)).toBe("0%");
  });
});
