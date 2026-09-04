<script setup lang="ts">
import { onMounted } from "vue";
import type { Component } from "vue";
import AppHeader from "./components/AppHeader.vue";
import StatusBar from "./components/StatusBar.vue";
import HistoryView from "./views/HistoryView.vue";
import SettingsView from "./views/SettingsView.vue";
import WorkbenchView from "./views/WorkbenchView.vue";
import { useAppStore, type AppView } from "./stores/app";

const app = useAppStore();

const views: Record<AppView, Component> = {
  workbench: WorkbenchView,
  history: HistoryView,
  settings: SettingsView,
};

onMounted(() => app.init());
</script>

<template>
  <div class="flex h-full flex-col">
    <AppHeader />
    <main class="min-h-0 flex-1">
      <component :is="views[app.view]" />
    </main>
    <StatusBar />
  </div>
</template>
