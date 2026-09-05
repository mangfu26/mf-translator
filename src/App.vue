<script setup lang="ts">
import { onMounted, ref } from "vue";
import type { Component } from "vue";
import AppHeader from "./components/AppHeader.vue";
import StatusBar from "./components/StatusBar.vue";
import HistoryView from "./views/HistoryView.vue";
import SettingsView from "./views/SettingsView.vue";
import WorkbenchView from "./views/WorkbenchView.vue";
import QuickWindowView from "./views/QuickWindowView.vue";
import { currentWindowLabel } from "./lib/window";
import { useAppStore, type AppView } from "./stores/app";

const app = useAppStore();
const isQuick = ref(false);

const views: Record<AppView, Component> = {
  workbench: WorkbenchView,
  history: HistoryView,
  settings: SettingsView,
};

onMounted(async () => {
  const label = await currentWindowLabel();
  isQuick.value = label === "quick";
  app.init();
});
</script>

<template>
  <!-- 快捷小窗：独立精简 UI，无主窗口外壳 -->
  <div v-if="isQuick" class="h-full overflow-hidden">
    <QuickWindowView />
  </div>

  <!-- 主窗口：现有工作台/历史/设置 -->
  <div v-else class="flex h-full flex-col">
    <AppHeader />
    <main class="min-h-0 flex-1">
      <component :is="views[app.view]" />
    </main>
    <StatusBar />
  </div>
</template>
