<template>
  <div>
    <template v-if="serverData.status !== 'suspended'">
      <ServerLabelGame
        v-if="showGameLabel"
        :game="serverData.game"
        :mc-version="serverData.mc_version ?? ''"
        :is-link="linked"
      />
      <ServerLabelPlatform
        :loader="serverData.loader"
        :loader-version="serverData.loader_version ?? ''"
        :no-separator="column"
        :is-link="linked"
      />
      <ServerLabelUrl
        v-if="serverData.net?.domain"
        :subdomain="serverData.net.domain"
        :no-separator="column"
        :is-link="linked"
      />
      <ServerLabelUptime
        v-if="uptimeSeconds"
        :uptime-seconds="uptimeSeconds"
        :no-separator="column"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import ServerLabelGame from "./ServerLabelGame.vue";
import ServerLabelPlatform from "./ServerLabelPlatform.vue";
import ServerLabelUptime from "./ServerLabelUptime.vue";
import ServerLabelUrl from "./ServerLabelUrl.vue";

interface ServerLabelsProps {
  serverData: Record<string, any>;
  showGameLabel: boolean;
  showLoaderLabel: boolean;
  uptimeSeconds?: number;
  column?: boolean;
  linked?: boolean;
}

defineProps<ServerLabelsProps>();
</script>
