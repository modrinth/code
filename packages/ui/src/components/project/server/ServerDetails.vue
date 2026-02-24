<script setup lang="ts">
import ServerModpackContent from './ServerModpackContent.vue'
import ServerOnlinePlayers from './ServerOnlinePlayers.vue'
import ServerPing from './ServerPing.vue'
import ServerRecentPlays from './ServerRecentPlays.vue'
import ServerRegion from './ServerRegion.vue'

defineProps<{
	onlinePlayers?: number
	recentPlays?: number
	region?: string
	ping?: number
	modpackContent?: {
		name: string
		icon?: string
		link?: string
	}
}>()
</script>
<template>
	<div class="empty:hidden flex items-center gap-2">
		<ServerOnlinePlayers v-if="onlinePlayers !== undefined" :online="onlinePlayers" />
		<ServerRecentPlays v-if="recentPlays !== undefined" :recent-plays="recentPlays" />
		<ServerRegion v-if="region" :region="region" />
		<ServerPing v-if="ping" :ping="ping" />
		<ServerModpackContent
			v-if="modpackContent"
			:name="modpackContent.name"
			:icon="modpackContent.icon"
			:link="modpackContent.link"
		/>
	</div>
</template>
