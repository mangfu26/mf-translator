<script setup lang="ts">
import { cn } from "../../lib/utils";
import { TRANSLATION_MODES, type TranslationModeId } from "../../constants/translationModes";

const model = defineModel<TranslationModeId>({ required: true });
defineProps<{ compact?: boolean }>();
</script>

<template>
  <div class="flex items-center gap-1">
    <button
      v-for="mode in TRANSLATION_MODES"
      :key="mode.id"
      type="button"
      :class="
        cn(
          'rounded-md transition-colors',
          compact ? 'px-2 py-1 text-[11px]' : 'px-2.5 py-1.5 text-xs',
          model === mode.id
            ? 'bg-primary text-primary-foreground'
            : 'text-muted-foreground hover:bg-accent hover:text-foreground',
        )
      "
      @click="model = mode.id"
    >
      {{ mode.label }}
    </button>
  </div>
</template>
