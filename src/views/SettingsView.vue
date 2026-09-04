<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import UiButton from "../components/ui/UiButton.vue";
import UiInput from "../components/ui/UiInput.vue";
import { errorMessage } from "../services/ipc";
import { checkUpdate, type UpdateStatus } from "../services/update";
import { useAppStore } from "../stores/app";
import { useSettingsStore } from "../stores/settings";
import { t } from "../i18n";
import type { ProtocolId } from "../services/config";

const messages = t();
const settings = useSettingsStore();
const app = useAppStore();

const protocolOptions: { value: ProtocolId; label: string }[] = [
  { value: "chatCompletions", label: "Chat Completions" },
  { value: "responses", label: "Responses API" },
];

const form = reactive({
  presetId: "custom",
  name: "",
  baseUrl: "",
  model: "",
  protocol: "chatCompletions" as ProtocolId,
  apiKey: "",
});

onMounted(async () => {
  await settings.init();
  applyCurrent();
});

const currentId = computed(() => settings.current?.id ?? null);
const presetModels = computed(
  () => settings.presets.find((p) => p.id === form.presetId)?.defaultModels ?? [],
);

function applyCurrent() {
  const current = settings.current;
  if (!current) return;
  form.presetId = "custom";
  form.name = current.name;
  form.baseUrl = current.baseUrl;
  form.model = current.model;
  form.protocol = current.protocol;
  form.apiKey = "";
}

function applyPreset(id: string) {
  if (id === "custom") return;
  const preset = settings.presets.find((p) => p.id === id);
  if (!preset) return;
  form.name = preset.name;
  form.baseUrl = preset.baseUrl;
  form.protocol = preset.defaultProtocol;
  form.model = preset.defaultModels[0] ?? "";
  form.apiKey = "";
}

async function onTest() {
  await settings.runTest({
    providerId: currentId.value,
    baseUrl: form.baseUrl,
    model: form.model,
    protocol: form.protocol,
    apiKey: form.apiKey.trim() ? form.apiKey.trim() : null,
  });
}

async function onSave() {
  await settings.save(
    {
      id: currentId.value,
      name: form.name,
      baseUrl: form.baseUrl,
      model: form.model,
      protocol: form.protocol,
    },
    form.apiKey.trim() ? form.apiKey.trim() : null,
  );
}

function onPresetChange(value: string) {
  form.presetId = value;
  applyPreset(value);
}

type UpdateState = "idle" | "checking" | "latest" | "available" | "error";
const updateState = ref<UpdateState>("idle");
const updateInfo = ref<UpdateStatus | null>(null);
const updateError = ref("");

async function onCheckUpdate() {
  updateState.value = "checking";
  updateError.value = "";
  try {
    updateInfo.value = await checkUpdate();
    updateState.value = updateInfo.value.updateAvailable ? "available" : "latest";
  } catch (err) {
    updateState.value = "error";
    updateError.value = errorMessage(err);
  }
}
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-xl flex-col gap-5 overflow-y-auto p-6">
    <div>
      <h2 class="text-base font-semibold">{{ messages.settings.title }}</h2>
      <p class="mt-1 text-xs leading-relaxed text-muted-foreground">
        {{ messages.settings.hint }}
      </p>
    </div>

    <label class="block">
      <div class="mb-1.5 text-xs font-medium text-muted-foreground">
        {{ messages.settings.preset }}
      </div>
      <UiSelect
        :model-value="form.presetId"
        :options="[
          { value: 'custom', label: messages.settings.custom },
          ...settings.presets.map((p) => ({ value: p.id, label: p.name })),
        ]"
        @update:model-value="onPresetChange"
      />
    </label>

    <label class="block">
      <div class="mb-1.5 text-xs font-medium text-muted-foreground">
        {{ messages.settings.name }}
      </div>
      <UiInput v-model="form.name" :placeholder="messages.settings.custom" />
    </label>

    <label class="block">
      <div class="mb-1.5 text-xs font-medium text-muted-foreground">
        {{ messages.settings.baseUrl }}
      </div>
      <UiInput v-model="form.baseUrl" placeholder="https://api.deepseek.com/v1" />
    </label>

    <label class="block">
      <div class="mb-1.5 text-xs font-medium text-muted-foreground">
        {{ messages.settings.model }}
      </div>
      <UiInput v-model="form.model" list="preset-models" placeholder="deepseek-chat" />
      <datalist id="preset-models">
        <option v-for="m in presetModels" :key="m" :value="m" />
      </datalist>
    </label>

    <div>
      <div class="mb-1.5 text-xs font-medium text-muted-foreground">
        {{ messages.settings.protocol }}
      </div>
      <div class="inline-flex rounded-lg border border-border p-0.5">
        <button
          v-for="option in protocolOptions"
          :key="option.value"
          type="button"
          class="rounded-md px-3 py-1.5 text-xs transition-colors"
          :class="
            form.protocol === option.value
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:bg-accent'
          "
          @click="form.protocol = option.value"
        >
          {{ option.label }}
        </button>
      </div>
    </div>

    <label class="block">
      <div class="mb-1.5 text-xs font-medium text-muted-foreground">
        {{ messages.settings.apiKey }}
      </div>
      <UiInput
        v-model="form.apiKey"
        type="password"
        :placeholder="messages.settings.apiKeyPlaceholder"
      />
      <p
        v-if="settings.current?.hasApiKey"
        class="mt-1.5 text-xs leading-relaxed text-muted-foreground/80"
      >
        {{ messages.settings.apiKeySavedHint }}
      </p>
    </label>

    <div
      v-if="settings.test.status === 'error' || settings.saveError"
      class="rounded-lg border border-red-500/40 bg-red-500/10 p-3 text-xs leading-relaxed text-red-500"
    >
      {{ settings.saveError || settings.test.message }}
    </div>
    <div
      v-else-if="settings.test.status === 'ok'"
      class="rounded-lg border border-green-500/40 bg-green-500/10 p-3 text-xs text-green-600 dark:text-green-400"
    >
      {{ settings.test.message }}
    </div>

    <div class="flex items-center gap-2">
      <UiButton
        :disabled="!form.baseUrl || !form.model"
        :loading="settings.test.status === 'testing'"
        @click="onTest()"
      >
        {{
          settings.test.status === "testing" ? messages.settings.testing : messages.settings.test
        }}
      </UiButton>
      <UiButton
        variant="primary"
        :disabled="!form.baseUrl || !form.model"
        :loading="settings.saving"
        @click="onSave()"
      >
        {{ settings.saving ? messages.settings.saving : messages.settings.save }}
      </UiButton>
      <span v-if="settings.savedFlash" class="text-xs text-green-600 dark:text-green-400">
        {{ messages.settings.saved }}
      </span>
    </div>

    <section class="mt-2 rounded-xl border border-border bg-card p-4">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-semibold">{{ messages.update.title }}</h3>
        <span class="text-xs text-muted-foreground">
          {{ messages.update.current }} v{{ app.engine.appVersion || "–" }}
        </span>
      </div>

      <div
        v-if="updateState === 'available' && updateInfo"
        class="mt-3 rounded-lg border border-primary/40 bg-primary/10 p-3"
      >
        <div class="text-xs font-semibold text-primary">
          {{ messages.update.available }}：v{{ updateInfo.latestVersion }}
        </div>
        <p v-if="updateInfo.notes" class="mt-1 text-xs leading-relaxed text-muted-foreground">
          {{ updateInfo.notes }}
        </p>
        <UiButton
          variant="primary"
          size="sm"
          class="mt-2.5"
          @click="openUrl(updateInfo.downloadUrl)"
        >
          {{ messages.update.download }}
        </UiButton>
      </div>
      <p
        v-else-if="updateState === 'latest'"
        class="mt-3 text-xs text-green-600 dark:text-green-400"
      >
        {{ messages.update.latest }}
      </p>
      <div
        v-else-if="updateState === 'error'"
        class="mt-3 rounded-lg border border-red-500/40 bg-red-500/10 p-3 text-xs leading-relaxed text-red-500"
      >
        {{ messages.update.failed }}：{{ updateError }}
      </div>

      <div class="mt-3">
        <UiButton :loading="updateState === 'checking'" @click="onCheckUpdate()">
          {{ updateState === "checking" ? messages.update.checking : messages.update.check }}
        </UiButton>
      </div>
    </section>
  </div>
</template>
