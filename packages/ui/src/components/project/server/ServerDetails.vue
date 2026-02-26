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
	hideOnlinePlayersLabel?: boolean
	hideRecentPlaysLabel?: boolean
}>()
</script>
<template>
	<div class="empty:hidden flex items-center gap-2">
		<ServerOnlinePlayers
			v-if="onlinePlayers !== undefined"
			:online="onlinePlayers"
			:hide-label="hideOnlinePlayersLabel"
		/>
		<ServerRecentPlays
			v-if="recentPlays !== undefined"
			:recent-plays="recentPlays"
			:hide-label="hideRecentPlaysLabel"
		/>
		<ServerRegion v-if="region" :region="region" />
		<ServerPing v-if="ping !== undefined" :ping="ping" />
		<ServerModpackContent
			v-if="modpackContent"
			:name="modpackContent.name"
			:icon="modpackContent.icon"
			:link="modpackContent.link"
		/>
	</div>
</template>
