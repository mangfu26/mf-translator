import { defineStore } from "pinia";
import {
  getPromptTemplates,
  resetPromptTemplate,
  savePromptTemplate,
  type PromptTemplateDto,
} from "../services/prompt";
import { errorMessage } from "../services/ipc";

export const usePromptStore = defineStore("prompt", {
  state: () => ({
    templates: [] as PromptTemplateDto[],
    loaded: false,
    saving: false,
    error: "",
  }),
  actions: {
    async init() {
      if (this.loaded) return;
      await this.reload();
    },
    async reload() {
      try {
        this.templates = await getPromptTemplates();
        this.loaded = true;
        this.error = "";
      } catch (err) {
        this.error = errorMessage(err);
      }
    },
    async save(key: string, content: string) {
      this.saving = true;
      this.error = "";
      try {
        await savePromptTemplate(key, content);
        await this.reload();
      } catch (err) {
        this.error = errorMessage(err);
      } finally {
        this.saving = false;
      }
    },
    async reset(key?: string) {
      this.error = "";
      try {
        this.templates = await resetPromptTemplate(key);
      } catch (err) {
        this.error = errorMessage(err);
      }
    },
  },
});
