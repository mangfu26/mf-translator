<script setup lang="ts">
withDefaults(
  defineProps<{
    type?: string;
    placeholder?: string;
    disabled?: boolean;
    list?: string;
  }>(),
  { type: "text", placeholder: "", disabled: false, list: undefined },
);

const model = defineModel<string>({ required: true });
</script>

<template>
  <input
    v-model="model"
    :type="type"
    :placeholder="placeholder"
    :disabled="disabled"
    :list="list"
    autocomplete="off"
    spellcheck="false"
    class="h-9 w-full rounded-lg border border-border bg-card px-3 text-sm outline-none transition-colors placeholder:text-muted-foreground/60 focus:border-primary disabled:opacity-50"
  />
</template>
