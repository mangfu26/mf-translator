import { beforeEach, describe, expect, it, vi } from "vitest";

// IPC 契约测试：Tauri 按参数名匹配 command 入参，命名错位只会在运行时报错。
// 这里固化每个前端调用与 Rust command 的「命令名 + 参数键名」契约。

vi.mock("@tauri-apps/api/core", () => ({
  Channel: class {
    onmessage?: (message: unknown) => void;
  },
}));

vi.mock("./ipc", () => ({
  invoke: vi.fn(),
  errorMessage: vi.fn((err: unknown) => String(err)),
}));

import { invoke } from "./ipc";
import { getProviderConfig, listPresets, saveProviderConfig, testConnection } from "./config";
import { cancelTranslation, startTranslation } from "./translate";
import { clearHistory, deleteHistoryItem, listHistory } from "./history";

const invokeMock = vi.mocked(invoke);

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue({});
});

describe("IPC 参数契约", () => {
  it("save_provider_config 使用 input + apiKey 参数名", async () => {
    const input = {
      name: "DeepSeek",
      baseUrl: "https://api.deepseek.com/v1",
      model: "deepseek-chat",
      protocol: "chatCompletions" as const,
    };
    await saveProviderConfig(input, "sk-1");
    expect(invokeMock).toHaveBeenCalledWith("save_provider_config", {
      input,
      apiKey: "sk-1",
    });
  });

  it("save_provider_config 空键传 null（清除语义）", async () => {
    await saveProviderConfig(
      { name: "x", baseUrl: "https://x.com/v1", model: "m", protocol: "responses" },
      null,
    );
    expect(invokeMock).toHaveBeenCalledWith(
      "save_provider_config",
      expect.objectContaining({ apiKey: null }),
    );
  });

  it("test_connection / list_presets / get_provider_config 参数形状", async () => {
    await testConnection({
      providerId: "p1",
      baseUrl: "https://x.com/v1",
      model: "m",
      protocol: "responses",
      apiKey: null,
    });
    expect(invokeMock).toHaveBeenCalledWith("test_connection", {
      input: {
        providerId: "p1",
        baseUrl: "https://x.com/v1",
        model: "m",
        protocol: "responses",
        apiKey: null,
      },
    });

    await listPresets();
    expect(invokeMock).toHaveBeenCalledWith("list_presets");

    await getProviderConfig();
    expect(invokeMock).toHaveBeenCalledWith("get_provider_config");
  });

  it("translate_text 携带 request 与 channel", async () => {
    invokeMock.mockResolvedValueOnce("task-1");
    const taskId = await startTranslation(
      {
        model: "m",
        sourceText: "hi",
        targetLanguage: "简体中文",
        sourceLanguage: null,
      },
      () => {},
    );
    expect(taskId).toBe("task-1");
    const [cmd, args] = invokeMock.mock.calls[0];
    expect(cmd).toBe("translate_text");
    expect(args).toMatchObject({
      request: { model: "m", sourceText: "hi", targetLanguage: "简体中文" },
    });
    expect(args?.channel).toBeTypeOf("object");
  });

  it("cancel_translation 使用 taskId 参数名", async () => {
    await cancelTranslation("task-1");
    expect(invokeMock).toHaveBeenCalledWith("cancel_translation", { taskId: "task-1" });
  });

  it("history 三命令的命令名与参数名", async () => {
    await listHistory("关键词");
    expect(invokeMock).toHaveBeenCalledWith("list_history", {
      query: "关键词",
      limit: 200,
    });

    await deleteHistoryItem(7);
    expect(invokeMock).toHaveBeenCalledWith("delete_history_item", { id: 7 });

    await clearHistory();
    expect(invokeMock).toHaveBeenCalledWith("clear_history");
  });
});
