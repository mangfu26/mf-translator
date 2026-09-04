<script setup lang="ts">
import { useAppStore } from "../stores/app";
import { t } from "../i18n";

const app = useAppStore();
const messages = t();
</script>

<template>
  <footer
    class="flex h-8 shrink-0 items-center justify-between border-t border-border px-4 text-[11px] text-muted-foreground"
  >
    <span class="flex items-center gap-1.5">
      <span
        class="size-1.5 rounded-full"
        :class="app.engine.ready ? 'bg-success' : 'bg-border'"
        :title="app.engine.ready ? messages.engine.ready : messages.engine.offline"
      />
      {{ messages.status.platform }}
    </span>
    <span>{{ messages.status.version }} {{ app.engine.appVersion || "–" }}</span>
  </footer>
</template>
