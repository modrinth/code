<template>
  <div
    :aria-label="`Server is ${getStatusText}`"
    class="relative inline-flex select-none items-center"
    @mouseenter="isExpanded = true"
    @mouseleave="isExpanded = false"
  >
    <div
      :class="`h-4 w-4 rounded-full transition-all duration-300 ease-in-out ${getStatusClass.main}`"
    >
      <div
        :class="`absolute inline-flex h-4 w-4 animate-ping rounded-full ${getStatusClass.bg}`"
      ></div>
    </div>
    <div
      :class="`absolute -left-4 flex w-auto items-center gap-2 rounded-full px-2 py-1 transition-all duration-150 ease-in-out ${getStatusClass.bg} ${
        isExpanded ? 'translate-x-2 scale-100 opacity-100' : 'translate-x-0 scale-90 opacity-0'
      }`"
    >
      <div class="h-3 w-3 rounded-full"></div>
      <span
        class="origin-left whitespace-nowrap text-sm font-semibold text-contrast transition-all duration-[200ms] ease-in-out"
        :class="`${isExpanded ? 'translate-x-0 scale-100' : '-translate-x-1 scale-x-75'}`"
      >
        {{ getStatusText }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type { ServerState } from "~/types/servers";

const props = defineProps<{
  state: ServerState;
}>();

const isExpanded = ref(false);

const getStatusClass = computed(() => {
  switch (props.state) {
    case "running":
      return { main: "bg-brand", bg: "bg-bg-green" };
    case "stopped":
      return { main: "", bg: "" };
    case "crashed":
      return { main: "bg-brand-red", bg: "bg-bg-red" };
    default:
      return { main: "", bg: "" };
  }
});

const getStatusText = computed(() => {
  switch (props.state) {
    case "running":
      return "Running";
    case "stopped":
      return "";
    case "crashed":
      return "Crashed";
    default:
      return "Unknown";
  }
});
</script>
