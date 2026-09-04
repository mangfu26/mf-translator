import { defineStore } from "pinia";
import {
  getProviderConfig,
  listPresets,
  saveProviderConfig,
  testConnection,
  type ProviderConfigInput,
  type ProviderConfigView,
  type ProviderPresetDto,
  type TestConnectionInput,
} from "../services/config";
import { errorMessage } from "../services/ipc";

interface TestState {
  status: "idle" | "testing" | "ok" | "error";
  message: string;
}

export const useSettingsStore = defineStore("settings", {
  state: () => ({
    presets: [] as ProviderPresetDto[],
    current: null as ProviderConfigView | null,
    loaded: false,
    saving: false,
    saveError: "",
    savedFlash: false,
    test: { status: "idle", message: "" } as TestState,
  }),
  actions: {
    async init() {
      if (this.loaded) return;
      try {
        this.presets = await listPresets();
        this.current = await getProviderConfig();
        this.loaded = true;
      } catch (err) {
        this.saveError = errorMessage(err);
      }
    },
    async save(input: ProviderConfigInput, apiKey: string | null) {
      this.saving = true;
      this.saveError = "";
      this.savedFlash = false;
      try {
        this.current = await saveProviderConfig(input, apiKey);
        this.saving = false;
        this.savedFlash = true;
        setTimeout(() => {
          this.savedFlash = false;
        }, 2000);
      } catch (err) {
        this.saving = false;
        this.saveError = errorMessage(err);
      }
    },
    async runTest(input: TestConnectionInput) {
      this.test = { status: "testing", message: "" };
      try {
        const message = await testConnection(input);
        this.test = { status: "ok", message };
      } catch (err) {
        this.test = { status: "error", message: errorMessage(err) };
      }
    },
  },
});
