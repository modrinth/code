<template>
	<div>
		<ServerGameLabel
			v-if="showGameLabel"
			:game="serverData.game"
			:mc-version="serverData.mc_version ?? ''"
			:is-link="linked"
		/>
		<ServerLoaderLabel
			:loader="serverData.loader"
			:loader-version="serverData.loader_version ?? ''"
			:no-separator="column"
			:is-link="linked"
		/>
		<ServerSubdomainLabel
			v-if="serverData.net?.domain"
			:subdomain="serverData.net.domain"
			:no-separator="column"
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
import ServerSubdomainLabel from './ServerSubdomainLabel.vue'
import ServerUptimeLabel from './ServerUptimeLabel.vue'

interface ServerInfoLabelsProps {
	serverData: Record<string, any>
	showGameLabel: boolean
	showLoaderLabel: boolean
	uptimeSeconds?: number
	column?: boolean
	linked?: boolean
}

defineProps<ServerInfoLabelsProps>()
</script>
