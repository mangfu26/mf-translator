<script setup lang="ts">
import { computed } from "vue";
import { cn } from "../../lib/utils";

const props = withDefaults(
  defineProps<{
    variant?: "primary" | "outline" | "ghost" | "danger";
    size?: "sm" | "md";
    disabled?: boolean;
    loading?: boolean;
  }>(),
  { variant: "outline", size: "md", disabled: false, loading: false },
);

const classes = computed(() =>
  cn(
    "inline-flex items-center justify-center gap-1.5 rounded-lg font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50",
    props.size === "sm" ? "h-7 px-2.5 text-xs" : "h-9 px-4 text-sm",
    props.variant === "primary" && "bg-primary text-primary-foreground hover:opacity-90",
    props.variant === "outline" && "border border-border hover:bg-accent",
    props.variant === "ghost" && "text-muted-foreground hover:bg-accent hover:text-foreground",
    props.variant === "danger" && "border border-border text-red-500 hover:bg-red-500/10",
  ),
);
</script>

<template>
  <button type="button" :class="classes" :disabled="disabled || loading">
    <span
      v-if="loading"
      class="size-3 animate-spin rounded-full border-2 border-current border-t-transparent"
    />
    <slot />
  </button>
</template>
