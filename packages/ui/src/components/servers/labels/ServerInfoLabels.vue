<template>
  <div>
    <ServerGameLabel
      v-if="showGameLabel"
      :game="serverData.game"
      :mc-version="serverData.mc_version ?? ''"
      :is-link="linked"
      :link-component="linkComponent"
      :server-id="serverId"
    />
    <ServerLoaderLabel
      :loader="serverData.loader"
      :loader-version="serverData.loader_version ?? ''"
      :no-separator="column"
      :is-link="linked"
      :tags="tags"
      :link-component="linkComponent"
      :server-id="serverId"
    />
    <ServerSubdomainLabel
      v-if="serverData.net?.domain"
      :subdomain="serverData.net.domain"
      :no-separator="column"
      :is-link="linked"
      :server-id="serverId"
    />
    <ServerUptimeLabel
      v-if="uptimeSeconds"
      :uptime-seconds="uptimeSeconds"
      :no-separator="column"
    />
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import {
  ServerGameLabel,
  ServerLoaderLabel,
  ServerSubdomainLabel,
  ServerUptimeLabel,
  type LoaderTag,
} from '@modrinth/ui'

interface ServerInfoLabelsProps {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  serverData: Record<string, any>
  showGameLabel: boolean
  showLoaderLabel: boolean
  uptimeSeconds?: number
  column?: boolean
  linked?: boolean
  serverId: string
  linkComponent?: Component
  tags: {
    loaders: LoaderTag[]
  }
}

defineProps<ServerInfoLabelsProps>()
</script>
