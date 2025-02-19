<template>
  <div
    :aria-label="`Server is ${getStatusText(state)}`"
    class="relative inline-flex select-none items-center"
    @mouseenter="isExpanded = true"
    @mouseleave="isExpanded = false"
  >
    <div
      :class="[
        'h-4 w-4 rounded-full transition-all duration-300 ease-in-out',
        getStatusClass(state).main,
      ]"
    >
      <div
        :class="[
          'absolute inline-flex h-4 w-4 animate-ping rounded-full',
          getStatusClass(state).bg,
        ]"
      ></div>
    </div>
    <div
      :class="[
        'absolute -left-4 flex w-auto items-center gap-2 rounded-full px-2 py-1 transition-all duration-150 ease-in-out',
        getStatusClass(state).bg,
        isExpanded ? 'translate-x-2 scale-100 opacity-100' : 'translate-x-0 scale-90 opacity-0',
      ]"
    >
      <div class="h-3 w-3 rounded-full"></div>
      <span
        :class="[
          'origin-left whitespace-nowrap text-sm font-semibold text-contrast transition-all duration-[200ms] ease-in-out',
          isExpanded ? 'translate-x-0 scale-100' : '-translate-x-1 scale-x-75',
        ]"
      >
        {{ getStatusText(state) }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import type { ServerState } from "~/types/servers";

const STATUS_CLASSES = {
  running: { main: "bg-brand", bg: "bg-bg-green" },
  stopped: { main: "", bg: "" },
  crashed: { main: "bg-brand-red", bg: "bg-bg-red" },
  unknown: { main: "", bg: "" },
} as const;

const STATUS_TEXTS = {
  running: "Running",
  stopped: "",
  crashed: "Crashed",
  unknown: "Unknown",
} as const;

defineProps<{
  state: ServerState;
}>();

const isExpanded = ref(false);

function getStatusClass(state: ServerState) {
  return STATUS_CLASSES[state] ?? STATUS_CLASSES.unknown;
}

function getStatusText(state: ServerState) {
  return STATUS_TEXTS[state] ?? STATUS_TEXTS.unknown;
}
</script>
