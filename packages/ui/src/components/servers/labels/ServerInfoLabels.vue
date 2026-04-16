<template>
	<div>
		<ServerPlayerCount
			v-if="showPlayerCount"
			:current-players="serverData.players.current"
			:max-players="serverData.players.max"
			:online="serverData.online"
		/>
		<ServerGameLabel
			v-if="showGameLabel"
			:game="serverData.game"
			:mc-version="serverData.mc_version ?? ''"
			:no-separator="column || !showPlayerCount"
			:is-link="linked"
		/>
		<ServerLoaderLabel
			v-if="showLoaderLabel"
			:loader="serverData.loader"
			:loader-version="serverData.loader_version ?? ''"
			:no-separator="column || !showGameLabel"
			:is-link="linked"
		/>
		<ServerSubdomainLabel
			v-if="serverData.net?.domain"
			:subdomain="serverData.net.domain"
			:server-id="serverId"
			:no-separator="column || (!showLoaderLabel && !showGameLabel)"
			:is-link="linked"
		/>
		<ServerUptimeLabel
			v-if="uptimeSeconds"
			:uptime-seconds="uptimeSeconds"
			:no-separator="column"
		/>
	</div>
</template>

<script setup lang="ts">
import ServerGameLabel from './ServerGameLabel.vue'
import ServerLoaderLabel from './ServerLoaderLabel.vue'
import ServerPlayerCount from './ServerPlayerCount.vue'
import ServerSubdomainLabel from './ServerSubdomainLabel.vue'
import ServerUptimeLabel from './ServerUptimeLabel.vue'

interface ServerInfoLabelsProps {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	serverData: Record<string, any>
	serverId?: string
	showGameLabel: boolean
	showLoaderLabel: boolean
	showPlayerCount?: boolean
	uptimeSeconds?: number
	column?: boolean
	linked?: boolean
}

defineProps<ServerInfoLabelsProps>()
</script>
