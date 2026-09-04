<script setup lang="ts">
defineProps<{
  modelValue: string;
  options: { value: string; label: string }[];
  disabled?: boolean;
}>();

const emit = defineEmits<{ (e: "update:modelValue", value: string): void }>();

function onChange(event: Event) {
  emit("update:modelValue", (event.target as HTMLSelectElement).value);
}
</script>

<template>
  <select
    :value="modelValue"
    :disabled="disabled"
    class="h-9 rounded-lg border border-border bg-card px-2.5 text-sm outline-none transition-colors hover:bg-accent/40 focus:border-primary disabled:opacity-50"
    @change="onChange"
  >
    <option v-for="option in options" :key="option.value" :value="option.value">
      {{ option.label }}
    </option>
  </select>
</template>
