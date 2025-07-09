<template>
  <div>
    <UiServersServerGameLabel
      v-if="showGameLabel"
      :game="serverData.game"
      :mc-version="serverData.mc_version ?? ''"
      :is-link="linked"
    />
    <UiServersServerLoaderLabel
      :loader="serverData.loader"
      :loader-version="serverData.loader_version ?? ''"
      :no-separator="column"
      :is-link="linked"
    />
    <UiServersServerSubdomainLabel
      v-if="serverData.net?.domain"
      :subdomain="serverData.net.domain"
      :no-separator="column"
      :is-link="linked"
    />
    <UiServersServerUptimeLabel
      v-if="uptimeSeconds"
      :uptime-seconds="uptimeSeconds"
      :no-separator="column"
    />
  </div>
</template>

<script setup lang="ts">
interface ServerInfoLabelsProps {
  serverData: Record<string, any>;
  showGameLabel: boolean;
  showLoaderLabel: boolean;
  uptimeSeconds?: number;
  column?: boolean;
  linked?: boolean;
}

defineProps<ServerInfoLabelsProps>();
</script>
