<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import UiButton from "./ui/UiButton.vue";
import { usePromptStore } from "../stores/prompt";
import { t } from "../i18n";

const messages = t();
const promptStore = usePromptStore();

const KEY_LABELS: Record<string, string> = {
  "system.general": "通用",
  "system.academic": "学术",
  "system.colloquial": "口语",
  "system.polish": "润色",
  user: "用户消息",
};

/// 占位符及其说明；点击 chip 即复制到剪贴板
const PLACEHOLDER_INFO: { token: string; desc: string }[] = [
  { token: "{target_language}", desc: messages.prompt.placeholderTarget },
  { token: "{source_language}", desc: messages.prompt.placeholderSource },
  { token: "{source_text}", desc: messages.prompt.placeholderText },
];

const activeKey = ref("");
/// 各模板的草稿（切换时保留未保存的修改）
const drafts = reactive<Record<string, string>>({});
const savedFlash = ref(false);
const confirmingReset = ref(false);
const confirmingResetAll = ref(false);
const copiedChip = ref("");
let savedTimer: ReturnType<typeof setTimeout> | undefined;
let confirmTimer: ReturnType<typeof setTimeout> | undefined;
let copiedTimer: ReturnType<typeof setTimeout> | undefined;

async function copyPlaceholder(token: string) {
  await writeText(token);
  copiedChip.value = token;
  clearTimeout(copiedTimer);
  copiedTimer = setTimeout(() => {
    copiedChip.value = "";
  }, 1500);
}

onMounted(async () => {
  await promptStore.init();
  if (!activeKey.value && promptStore.templates.length) {
    activeKey.value = promptStore.templates[0]!.key;
  }
});

function syncDrafts(force = false) {
  for (const tpl of promptStore.templates) {
    if (force || drafts[tpl.key] === undefined) drafts[tpl.key] = tpl.content;
  }
}

const activeTemplate = computed(() =>
  promptStore.templates.find((tpl) => tpl.key === activeKey.value),
);

const activeDraft = computed({
  get: () => drafts[activeKey.value] ?? activeTemplate.value?.content ?? "",
  set: (value: string) => {
    drafts[activeKey.value] = value;
  },
});

const dirty = computed(
  () => activeTemplate.value !== undefined && activeDraft.value !== activeTemplate.value.content,
);

function switchTo(key: string) {
  activeKey.value = key;
  confirmingReset.value = false;
  if (drafts[key] === undefined) {
    const tpl = promptStore.templates.find((t) => t.key === key);
    drafts[key] = tpl?.content ?? "";
  }
}

async function saveCurrent() {
  await promptStore.save(activeKey.value, activeDraft.value);
  syncDrafts(true);
  savedFlash.value = true;
  clearTimeout(savedTimer);
  savedTimer = setTimeout(() => {
    savedFlash.value = false;
  }, 1500);
}

async function resetCurrent() {
  if (!confirmingReset.value) {
    confirmingReset.value = true;
    clearTimeout(confirmTimer);
    confirmTimer = setTimeout(() => {
      confirmingReset.value = false;
    }, 3000);
    return;
  }
  confirmingReset.value = false;
  await promptStore.reset(activeKey.value);
  syncDrafts(true);
}

async function resetAll() {
  if (!confirmingResetAll.value) {
    confirmingResetAll.value = true;
    clearTimeout(confirmTimer);
    confirmTimer = setTimeout(() => {
      confirmingResetAll.value = false;
    }, 3000);
    return;
  }
  confirmingResetAll.value = false;
  await promptStore.reset();
  syncDrafts(true);
}

onBeforeUnmount(() => {
  clearTimeout(savedTimer);
  clearTimeout(confirmTimer);
  clearTimeout(copiedTimer);
});
</script>

<template>
  <section class="rounded-xl border border-border bg-card p-4">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-semibold">{{ messages.prompt.title }}</h3>
      <span class="text-[11px] text-muted-foreground">{{ messages.prompt.advancedHint }}</span>
    </div>
    <p class="mt-1 text-xs leading-relaxed text-muted-foreground">{{ messages.prompt.hint }}</p>

    <div
      v-if="promptStore.error"
      class="mt-3 rounded-lg border border-red-500/40 bg-red-500/10 p-3 text-xs text-red-500"
    >
      {{ promptStore.error }}
    </div>

    <!-- 模板切换条（已修改的模板名带圆点标记） -->
    <div class="mt-3 flex flex-wrap items-center gap-1">
      <button
        v-for="tpl in promptStore.templates"
        :key="tpl.key"
        type="button"
        class="relative rounded-lg px-3 py-1.5 text-xs transition-colors"
        :class="
          activeKey === tpl.key
            ? 'bg-primary text-primary-foreground'
            : 'text-muted-foreground hover:bg-accent hover:text-foreground'
        "
        @click="switchTo(tpl.key)"
      >
        {{ KEY_LABELS[tpl.key] ?? tpl.key }}
        <span
          v-if="tpl.userModified"
          class="absolute right-1 top-1 size-1.5 rounded-full bg-amber-500"
          :title="messages.prompt.modified"
        />
      </button>
      <span class="flex-1" />
      <UiButton variant="ghost" size="sm" @click="resetAll()">
        {{ confirmingResetAll ? messages.prompt.confirmResetAll : messages.prompt.resetAll }}
      </UiButton>
    </div>

    <!-- 大编辑区（一次只编辑选中的一个模板） -->
    <textarea
      v-model="activeDraft"
      class="mt-3 h-80 w-full resize-y rounded-lg border border-border bg-muted/30 p-4 font-mono text-xs leading-relaxed outline-none select-text focus:border-primary"
      spellcheck="false"
    />

    <!-- 操作区：占位符说明（点击复制）+ 未保存提示 + 重置/保存 -->
    <div class="mt-2.5 flex flex-wrap items-center justify-between gap-2">
      <div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-muted-foreground/80">
        <span class="font-medium">{{ messages.prompt.placeholders }}</span>
        <span
          v-for="info in PLACEHOLDER_INFO"
          :key="info.token"
          class="inline-flex items-center gap-1"
        >
          <button
            type="button"
            :title="info.desc"
            class="rounded border border-border bg-muted px-1.5 py-0.5 font-mono transition-colors hover:border-primary hover:text-foreground"
            @click="copyPlaceholder(info.token)"
          >
            {{ copiedChip === info.token ? messages.prompt.copiedChip : info.token }}
          </button>
          <span>{{ info.desc }}</span>
        </span>
      </div>
      <div class="flex items-center gap-1.5">
        <span v-if="dirty" class="text-[11px] text-amber-500">
          {{ messages.prompt.unsaved }}
        </span>
        <span v-else-if="savedFlash" class="text-[11px] text-green-600 dark:text-green-400">
          {{ messages.prompt.saved }}
        </span>
        <UiButton variant="ghost" size="sm" @click="resetCurrent()">
          {{ confirmingReset ? messages.prompt.confirmReset : messages.prompt.reset }}
        </UiButton>
        <UiButton variant="primary" size="sm" :loading="promptStore.saving" @click="saveCurrent()">
          {{ messages.prompt.save }}
        </UiButton>
      </div>
    </div>
  </section>
</template>
