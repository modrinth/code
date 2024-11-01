<template>
  <div>
    <UiServersServerGameLabel
      v-if="showGameLabel"
      :game="serverData.game!"
      :mc-version="serverData.mc_version ?? ''"
    />
    <UiServersServerLoaderLabel
      v-if="showLoaderLabel"
      :loader="serverData.loader!"
      :loader-version="serverData.loader_version ?? ''"
      :no-separator="column"
    />
    <UiServersServerSubdomainLabel
      v-if="serverData.net.domain"
      :subdomain="serverData.net.domain"
      :no-separator="column"
    />
    <UiServersServerUptimeLabel
      v-if="uptimeSeconds"
      :uptime-seconds="uptimeSeconds"
      :no-separator="column"
    />
  </div>
</template>

<script setup lang="ts">
defineProps({
  serverData: {
    type: Object,
    required: true,
  },
  showGameLabel: {
    type: Boolean,
    required: true,
  },
  showLoaderLabel: {
    type: Boolean,
    required: true,
  },
  uptimeSeconds: {
    type: Number,
    required: false,
    default: 0,
  },
  column: {
    type: Boolean,
    required: false,
  },
});
</script>
