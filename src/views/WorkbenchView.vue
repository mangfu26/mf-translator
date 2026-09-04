<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import UiButton from "../components/ui/UiButton.vue";
import UiSelect from "../components/ui/UiSelect.vue";
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

const statusLine = computed(() => {
  if (translator.isStreaming) return messages.workbench.translating;
  if (translator.usage) {
    return `${translator.usage.promptTokens} + ${translator.usage.completionTokens} tokens`;
  }
  return "";
});

function onKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
    event.preventDefault();
    void translator.translate();
  }
}

function onGlobalKeydown(event: KeyboardEvent) {
  if (event.key === "Escape" && translator.isStreaming) {
    event.preventDefault();
    void translator.stop();
  }
}

onMounted(() => window.addEventListener("keydown", onGlobalKeydown));
onBeforeUnmount(() => {
  window.removeEventListener("keydown", onGlobalKeydown);
  clearTimeout(copiedTimer);
});
</script>

<template>
  <div class="flex h-full flex-col gap-3 p-4">
    <div class="flex items-center gap-2">
      <UiSelect v-model="translator.sourceLang" :options="sourceOptions" />
      <UiButton
        variant="ghost"
        size="sm"
        :title="messages.workbench.swap"
        @click="translator.swapLanguages()"
      >
        ⇄
      </UiButton>
      <UiSelect v-model="translator.targetLang" :options="targetOptions" />
      <span class="flex-1" />
      <UiButton
        v-if="!translator.isStreaming"
        variant="primary"
        :disabled="!translator.canTranslate"
        @click="translator.translate()"
      >
        {{ messages.workbench.translate }}
      </UiButton>
      <UiButton v-else variant="danger" @click="translator.stop()">
        {{ messages.workbench.stop }}
      </UiButton>
    </div>

    <div class="flex min-h-0 flex-1 flex-col gap-3">
      <textarea
        v-model="translator.input"
        :placeholder="messages.workbench.sourcePlaceholder"
        class="flex-1 resize-none rounded-xl border border-border bg-card p-4 text-sm leading-relaxed outline-none transition-colors select-text placeholder:text-muted-foreground/60 focus:border-primary"
        @keydown="onKeydown"
      />

      <div
        class="relative min-h-0 flex-1 overflow-y-auto rounded-xl border border-border bg-muted/40 p-4"
      >
        <p
          v-if="translator.output"
          class="whitespace-pre-wrap break-words text-sm leading-relaxed select-text"
        >
          {{ translator.output
          }}<span
            v-if="translator.isStreaming"
            class="ml-0.5 inline-block h-4 w-[2px] animate-pulse bg-primary align-text-bottom"
          />
        </p>
        <p v-else-if="translator.status !== 'error'" class="text-sm text-muted-foreground/70">
          {{ messages.workbench.outputPlaceholder }}
        </p>

        <div
          v-if="translator.status === 'error'"
          class="mt-3 rounded-lg border border-red-500/40 bg-red-500/10 p-3 text-xs leading-relaxed text-red-500"
        >
          <div class="font-semibold">{{ messages.workbench.streamError }}</div>
          <div class="mt-1">{{ translator.errorMsg }}</div>
        </div>
      </div>
    </div>

    <div class="flex items-center justify-between text-xs text-muted-foreground">
      <span>{{ statusLine }}</span>
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
</template>
