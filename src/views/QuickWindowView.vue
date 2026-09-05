<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import UiButton from "../components/ui/UiButton.vue";
import UiSelect from "../components/ui/UiSelect.vue";
import ModeSelect from "../components/ui/ModeSelect.vue";
import AlwaysOnTopToggle from "../components/AlwaysOnTopToggle.vue";
import { hideQuickWindow } from "../services/window";
import { LANGUAGES } from "../constants/languages";
import { t } from "../i18n";
import { useTranslatorStore } from "../stores/translator";

const messages = t();
const translator = useTranslatorStore();

const sourceOptions = [
  { value: "auto", label: messages.workbench.autoDetect },
  ...LANGUAGES.map((lang) => ({ value: lang, label: lang })),
];
const targetOptions = LANGUAGES.map((lang) => ({ value: lang, label: lang }));

const copied = ref(false);
let copiedTimer: ReturnType<typeof setTimeout> | undefined;

async function copyOutput() {
  if (!translator.output) return;
  await writeText(translator.output);
  copied.value = true;
  clearTimeout(copiedTimer);
  copiedTimer = setTimeout(() => {
    copied.value = false;
  }, 1500);
}

/// 点击译文区任意处一键复制；用户正在拖选文本时不触发
function onOutputClick() {
  const selection = window.getSelection();
  if (selection && selection.toString().length > 0) return;
  void copyOutput();
}

function onGlobalKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    void hideQuickWindow();
  }
  if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
    event.preventDefault();
    void translator.translate();
  }
}

onMounted(() => window.addEventListener("keydown", onGlobalKeydown));
onBeforeUnmount(() => {
  window.removeEventListener("keydown", onGlobalKeydown);
  clearTimeout(copiedTimer);
});
</script>

<template>
  <div class="flex h-full flex-col bg-background">
    <!-- 顶部拖动条（无边框窗口；整条可拖，按钮区保留点击） -->
    <div
      data-tauri-drag-region
      class="flex h-10 shrink-0 select-none items-center gap-2 border-b border-border px-3"
    >
      <span
        data-tauri-drag-region
        class="pointer-events-none flex-1 text-xs font-semibold text-muted-foreground"
      >
        MF 快捷翻译
      </span>
      <AlwaysOnTopToggle window-label="quick" />
      <button
        type="button"
        title="关闭"
        class="grid size-7 place-items-center rounded-lg text-muted-foreground transition-colors hover:bg-accent"
        @click="hideQuickWindow()"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M18 6 6 18M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div class="flex min-h-0 flex-1 flex-col gap-2.5 p-3">
      <div class="flex items-center gap-2">
        <ModeSelect v-model="translator.mode" compact />
      </div>

      <div class="flex items-center gap-2">
        <UiSelect v-model="translator.sourceLang" :options="sourceOptions" />
        <UiSelect v-model="translator.targetLang" :options="targetOptions" />
        <span class="flex-1" />
        <UiButton
          v-if="!translator.isStreaming"
          variant="primary"
          size="sm"
          :disabled="!translator.canTranslate"
          @click="translator.translate()"
        >
          {{ messages.workbench.translate }}
        </UiButton>
        <UiButton v-else variant="danger" size="sm" @click="translator.stop()">
          {{ messages.workbench.stop }}
        </UiButton>
      </div>

      <textarea
        v-model="translator.input"
        :placeholder="messages.workbench.sourcePlaceholder"
        class="h-28 resize-none rounded-lg border border-border bg-card p-3 text-sm leading-relaxed outline-none select-text placeholder:text-muted-foreground/60 focus:border-primary"
      />

      <div
        class="relative min-h-0 flex-1 overflow-y-auto rounded-lg border border-border bg-muted/40 p-3"
        :class="translator.output ? 'cursor-pointer' : ''"
        :title="translator.output ? messages.workbench.clickToCopy : ''"
        @click="onOutputClick"
      >
        <span
          v-if="copied"
          class="absolute right-2 top-2 rounded-md bg-primary px-2 py-1 text-[11px] text-primary-foreground shadow"
        >
          {{ messages.workbench.copied }}
        </span>
        <p
          v-if="translator.output"
          class="whitespace-pre-wrap break-words text-sm leading-relaxed select-text"
        >
          {{ translator.output
          }}<span
            v-if="translator.isStreaming"
            class="ml-0.5 inline-block h-4 w-px animate-pulse bg-primary align-text-bottom"
          />
        </p>
        <p v-else class="text-xs text-muted-foreground/70">
          {{ messages.workbench.outputPlaceholder }}
        </p>
        <div
          v-if="translator.status === 'error'"
          class="mt-2 rounded-md border border-red-500/40 bg-red-500/10 p-2 text-[11px] leading-relaxed text-red-500"
        >
          {{ translator.errorMsg }}
        </div>
      </div>

      <div class="flex items-center justify-between text-[11px] text-muted-foreground">
        <span v-if="translator.usage">
          {{ translator.usage.promptTokens }} + {{ translator.usage.completionTokens }} tokens
        </span>
        <span v-else />
        <div class="flex items-center gap-1">
          <UiButton variant="ghost" size="sm" @click="translator.clear()">
            {{ messages.workbench.clear }}
          </UiButton>
          <UiButton variant="ghost" size="sm" :disabled="!translator.output" @click="copyOutput()">
            {{ copied ? messages.workbench.copied : messages.workbench.copy }}
          </UiButton>
        </div>
      </div>
    </div>
  </div>
</template>
