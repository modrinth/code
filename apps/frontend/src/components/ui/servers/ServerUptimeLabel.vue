<template>
  <div
    v-if="uptimeSeconds"
    v-tooltip="'Server Uptime'"
    class="flex min-w-0 flex-row items-center gap-4"
    data-pyro-uptime
  >
    <div class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>

    <UiServersTimer class="flex size-5 shrink-0" />
    <div class="truncate text-sm font-semibold">
      {{ formattedUptime }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineProps } from "vue";

const props = defineProps<{
  uptimeSeconds: number;
}>();

const formattedUptime = computed(() => {
  const days = Math.floor(props.uptimeSeconds / (24 * 3600));
  const hours = Math.floor((props.uptimeSeconds % (24 * 3600)) / 3600);
  const minutes = Math.floor((props.uptimeSeconds % 3600) / 60);
  const seconds = props.uptimeSeconds % 60;

  let formatted = "";
  if (days > 0) {
    formatted += `${days}d `;
  }
  if (hours > 0 || days > 0) {
    formatted += `${hours}h `;
  }
  formatted += `${minutes}m ${seconds}s`;

  return formatted.trim();
});
</script>
