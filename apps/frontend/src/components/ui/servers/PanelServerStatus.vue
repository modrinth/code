<template>
  <div :class="`flex items-center gap-2 rounded-full px-2 py-1 ${getStatusClass}`">
    <span class="text-sm font-semibold">
      {{ getStatusText }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { ServerState } from "~/types/servers";

const props = defineProps<{
  state: ServerState;
}>();

const getStatusClass = computed(() => {
  switch (props.state) {
    case "running":
      return "bg-green-400/10 text-green-500";
    case "stopped":
      return "bg-button-border text-gray-500";
    case "crashed":
      return "bg-red-400/10 text-red-500";
    default:
      return "bg-button-border text-gray-500";
  }
});

const getStatusText = computed(() => {
  switch (props.state) {
    case "running":
      return "Running";
    case "stopped":
      return "Stopped";
    case "crashed":
      return "Crashed";
    default:
      return "Unknown";
  }
});
</script>
