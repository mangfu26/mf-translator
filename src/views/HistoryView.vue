<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import UiButton from "../components/ui/UiButton.vue";
import UiInput from "../components/ui/UiInput.vue";
import { formatRelativeTime } from "../lib/time";
import { t } from "../i18n";
import { useHistoryStore } from "../stores/history";

const messages = t();
const history = useHistoryStore();
const copiedId = ref<number | null>(null);
const confirmingClear = ref(false);
let copiedTimer: ReturnType<typeof setTimeout> | undefined;
let confirmTimer: ReturnType<typeof setTimeout> | undefined;
let searchTimer: ReturnType<typeof setTimeout> | undefined;

onMounted(() => {
  if (!history.items.length) void history.load();
});

onBeforeUnmount(() => {
  clearTimeout(copiedTimer);
  clearTimeout(confirmTimer);
  clearTimeout(searchTimer);
});

function onSearchInput() {
  clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    void history.load();
  }, 300);
}

async function copyItem(id: number, text: string) {
  await writeText(text);
  copiedId.value = id;
  clearTimeout(copiedTimer);
  copiedTimer = setTimeout(() => {
    copiedId.value = null;
  }, 1200);
}

function onClear() {
  if (!confirmingClear.value) {
    confirmingClear.value = true;
    confirmTimer = setTimeout(() => {
      confirmingClear.value = false;
    }, 3000);
    return;
  }
  confirmingClear.value = false;
  void history.clear();
}
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-xl flex-col gap-4 p-6">
    <div class="flex items-center justify-between">
      <h2 class="text-base font-semibold">{{ messages.history.title }}</h2>
      <UiButton v-if="history.items.length" variant="danger" size="sm" @click="onClear()">
        {{ confirmingClear ? messages.history.confirmClear : messages.history.clear }}
      </UiButton>
    </div>

    <UiInput
      v-model="history.query"
      :placeholder="messages.history.searchPlaceholder"
      @input="onSearchInput"
    />

    <p v-if="!history.items.length" class="mt-10 text-center text-sm text-muted-foreground/70">
      {{ messages.history.empty }}
    </p>

    <div class="min-h-0 flex-1 space-y-2 overflow-y-auto pr-1">
      <article
        v-for="item in history.items"
        :key="item.id"
        class="rounded-xl border border-border bg-card p-3.5"
      >
        <div class="flex items-center justify-between text-[11px] text-muted-foreground/80">
          <span>
            {{ formatRelativeTime(item.createdAt) }} · {{ item.provider }} /
            {{ item.model }}
          </span>
          <span v-if="item.promptTokens !== null" class="tabular-nums">
            {{ item.promptTokens }} + {{ item.completionTokens }} tokens
          </span>
        </div>
        <p class="mt-2 line-clamp-2 text-xs text-muted-foreground select-text">
          {{ item.sourceText }}
        </p>
        <p class="mt-1.5 text-sm leading-relaxed select-text">
          {{ item.translatedText }}
        </p>
        <div class="mt-2 flex items-center justify-end gap-1">
          <UiButton variant="ghost" size="sm" @click="copyItem(item.id, item.translatedText)">
            {{ copiedId === item.id ? messages.history.copied : messages.history.copy }}
          </UiButton>
          <UiButton variant="ghost" size="sm" @click="history.remove(item.id)">
            {{ messages.history.delete }}
          </UiButton>
        </div>
      </article>
    </div>
  </div>
</template>
