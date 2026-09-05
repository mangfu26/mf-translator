<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getWindowState, setAlwaysOnTop } from "../services/window";
import { cn } from "../lib/utils";

const props = defineProps<{ windowLabel: string }>();

const active = ref(false);
const loading = ref(false);

onMounted(async () => {
  try {
    const state = await getWindowState(props.windowLabel);
    active.value = state.alwaysOnTop;
  } catch {
    active.value = false;
  }
});

async function toggle() {
  loading.value = true;
  const next = !active.value;
  try {
    await setAlwaysOnTop(props.windowLabel, next);
    active.value = next;
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <button
    type="button"
    :title="active ? '已置顶' : '取消置顶'"
    :class="
      cn(
        'grid size-7 place-items-center rounded-lg transition-colors',
        active ? 'text-primary' : 'text-muted-foreground hover:bg-accent',
      )
    "
    :disabled="loading"
    @click="toggle"
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
      <path
        d="M12 2v8m0 0-3-3m3 3 3-3M4 14h16a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1Z"
      />
    </svg>
  </button>
</template>
