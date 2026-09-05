import { defineStore } from "pinia";
import { errorMessage } from "../services/ipc";
import type { Usage } from "../services/model";
import { cancelTranslation, startTranslation, type TranslationEvent } from "../services/translate";
import type { TranslationModeId } from "../constants/translationModes";
import { t } from "../i18n";
import { useAppStore } from "./app";
import { useSettingsStore } from "./settings";

type Status = "idle" | "streaming" | "error";

export const useTranslatorStore = defineStore("translator", {
  state: () => ({
    input: "",
    output: "",
    sourceLang: "auto",
    targetLang: "简体中文",
    mode: "general" as TranslationModeId,
    status: "idle" as Status,
    errorMsg: "",
    usage: null as Usage | null,
    taskId: "",
    /// 代际计数：旧任务的迟到事件不得污染新一轮输出。
    seq: 0,
  }),
  getters: {
    isStreaming: (state) => state.status === "streaming",
    canTranslate: (state) => state.input.trim().length > 0,
    /// 源语言为「自动检测」时不可交换（否则源/目标会相同）。
    canSwap: (state) => state.sourceLang !== "auto",
  },
  actions: {
    async translate() {
      if (!this.canTranslate || this.isStreaming) return;

      // 关键修复：settings store 默认未加载（重启后 current 为 null），
      // 若尚未初始化，先拉取后端持久化配置，避免误判「未配置供应商」。
      const settings = useSettingsStore();
      if (!settings.current && !settings.loaded) await settings.init();

      const config = settings.current;
      if (!config) {
        this.status = "error";
        this.errorMsg = t().workbench.notConfigured;
        useAppStore().setView("settings");
        return;
      }

      await this.stop();
      const gen = ++this.seq;
      this.output = "";
      this.usage = null;
      this.errorMsg = "";
      this.status = "streaming";

      const request = {
        model: config.model,
        sourceText: this.input,
        targetLanguage: this.targetLang,
        sourceLanguage: this.sourceLang === "auto" ? null : this.sourceLang,
        mode: this.mode,
      };

      try {
        const taskId = await startTranslation(request, (event) => {
          if (gen === this.seq) this.onEvent(event);
        });
        if (gen === this.seq) this.taskId = taskId;
      } catch (err) {
        if (gen !== this.seq) return;
        this.status = "error";
        this.errorMsg = errorMessage(err);
      }
    },
    onEvent(event: TranslationEvent) {
      if (event.type === "delta") {
        this.output += event.text;
      } else if (event.type === "finished") {
        this.usage = event.usage;
        if (this.status === "streaming") this.status = "idle";
        this.taskId = "";
      } else if (event.type === "failed") {
        if (event.error.code === "CANCELLED") {
          this.status = "idle";
          this.taskId = "";
          return;
        }
        this.status = "error";
        this.errorMsg = event.error.message;
        this.taskId = "";
      }
    },
    async stop() {
      this.seq += 1;
      if (this.taskId) {
        const id = this.taskId;
        this.taskId = "";
        try {
          await cancelTranslation(id);
        } catch {
          /* 任务可能已自然结束，忽略 */
        }
      }
      if (this.status === "streaming") this.status = "idle";
    },
    swapLanguages() {
      // 源语言处于「自动检测」时禁止交换：交换会让源/目标变得相同，令用户困惑
      if (this.sourceLang === "auto") return;
      const previous = this.sourceLang;
      this.sourceLang = this.targetLang;
      this.targetLang = previous;
    },
    clear() {
      void this.stop();
      this.input = "";
      this.output = "";
      this.errorMsg = "";
      this.usage = null;
      this.status = "idle";
    },
  },
});
