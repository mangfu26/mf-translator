import { describe, expect, it } from "vitest";
import { formatRelativeTime } from "./time";

describe("formatRelativeTime", () => {
  const now = Date.parse("2026-09-04T12:00:00+08:00");

  it("一分钟内显示刚刚", () => {
    expect(formatRelativeTime((now - 30_000) / 1000, now)).toBe("刚刚");
  });

  it("按分钟与小时递进", () => {
    expect(formatRelativeTime((now - 5 * 60_000) / 1000, now)).toBe("5 分钟前");
    expect(formatRelativeTime((now - 3 * 3_600_000) / 1000, now)).toBe("3 小时前");
  });

  it("超过一天显示绝对时间", () => {
    expect(formatRelativeTime((now - 2 * 86_400_000) / 1000, now)).toMatch(
      /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$/,
    );
  });
});
