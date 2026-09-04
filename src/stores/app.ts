import { defineStore } from "pinia";
import { fetchHealth, type HealthReport } from "../services/health";
import {
  applyTheme,
  loadThemePref,
  resolveEffectiveTheme,
  saveThemePref,
  watchSystemTheme,
  type ThemePref,
} from "../theme";
import { useHistoryStore } from "./history";

export type AppView = "workbench" | "history" | "settings";

interface EngineState {
  ready: boolean;
  appVersion: string;
  protocols: HealthReport["protocols"];
  checking: boolean;
}

const THEME_CYCLE: ThemePref[] = ["light", "dark", "system"];

export const useAppStore = defineStore("app", {
  state: () => ({
    view: "workbench" as AppView,
    theme: loadThemePref(),
    engine: {
      ready: false,
      appVersion: "",
      protocols: [],
      checking: false,
    } as EngineState,
  }),
  getters: {
    effectiveTheme: (state): "light" | "dark" =>
      resolveEffectiveTheme(state.theme, window.matchMedia("(prefers-color-scheme: dark)").matches),
  },
  actions: {
    init() {
      applyTheme(this.theme);
      watchSystemTheme(() => applyTheme(this.theme));
      void this.refreshHealth();
    },
    setView(view: AppView) {
      this.view = view;
      if (view === "history") {
        void useHistoryStore().load();
      } else if (view === "settings") {
        void useSettingsStoreBridge().init();
      }
    },
    setTheme(pref: ThemePref) {
      this.theme = pref;
      saveThemePref(pref);
      applyTheme(pref);
    },
    cycleTheme() {
      const next = THEME_CYCLE[(THEME_CYCLE.indexOf(this.theme) + 1) % THEME_CYCLE.length];
      this.setTheme(next);
    },
    async refreshHealth() {
      this.engine.checking = true;
      try {
        const report = await fetchHealth();
        this.engine = {
          ready: report.engineReady,
          appVersion: report.appVersion,
          protocols: report.protocols,
          checking: false,
        };
      } catch {
        this.engine.ready = false;
        this.engine.checking = false;
      }
    },
  },
});

/// 延迟导入避免循环依赖（settings store 会用到 app store）。
import { useSettingsStore } from "./settings";
function useSettingsStoreBridge() {
  return useSettingsStore();
}
