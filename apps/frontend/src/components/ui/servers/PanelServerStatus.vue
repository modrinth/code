<template>
  <div
    class="relative inline-flex items-center"
    @mouseenter="isExpanded = true"
    @mouseleave="isExpanded = false"
  >
    <div
      :class="`h-4 w-4 rounded-full transition-all duration-300 ease-in-out ${getStatusClass.main}`"
    >
      <div
        :class="`absolute inline-flex h-full w-full animate-ping rounded-full ${getStatusClass.bg}`"
      ></div>
    </div>
    <div
      :class="`absolute -left-2 flex w-auto items-center gap-2 rounded-full px-2 py-1 transition-all duration-300 ease-in-out ${getStatusClass.bg} ${
        isExpanded ? 'scale-x-100 opacity-100' : 'scale-x-0 opacity-0'
      }`"
    >
      <div class="h-3 w-3 rounded-full"></div>
      <span class="whitespace-nowrap text-sm font-semibold text-contrast">
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
      return { main: "bg-bg-red", bg: "bg-bg-red" };
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
