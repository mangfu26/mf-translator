<script setup lang="ts">
import ThemeToggle from "./ThemeToggle.vue";
import { useAppStore, type AppView } from "../stores/app";
import { t } from "../i18n";

const messages = t();
const app = useAppStore();

const navItems: { id: AppView; label: string }[] = [
  { id: "workbench", label: messages.nav.workbench },
  { id: "history", label: messages.nav.history },
  { id: "settings", label: messages.nav.settings },
];
</script>

<template>
  <header class="flex h-12 shrink-0 items-center justify-between border-b border-border px-4">
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-2.5">
        <div
          class="grid size-7 place-items-center rounded-lg bg-primary text-[13px] font-bold text-primary-foreground"
        >
          译
        </div>
        <div class="text-sm font-semibold">{{ messages.app.name }}</div>
      </div>
      <nav class="flex items-center gap-1">
        <button
          v-for="item in navItems"
          :key="item.id"
          type="button"
          class="rounded-lg px-3 py-1.5 text-xs transition-colors"
          :class="
            app.view === item.id
              ? 'bg-accent font-medium text-foreground'
              : 'text-muted-foreground hover:bg-accent/60 hover:text-foreground'
          "
          @click="app.setView(item.id)"
        >
          {{ item.label }}
        </button>
      </nav>
    </div>
    <ThemeToggle />
  </header>
</template>
