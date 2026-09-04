import { describe, expect, it } from "vitest";
import { cn } from "./utils";

describe("cn 工具函数", () => {
  it("合并冲突的 Tailwind 类并保留后者", () => {
    expect(cn("px-2 py-1", "px-4")).toBe("py-1 px-4");
  });

  it("过滤掉假值的条件类", () => {
    expect(cn("a", false && "b", undefined, "c")).toBe("a c");
  });
});
