<script setup lang="ts">
import ServerOnlinePlayers from './ServerOnlinePlayers.vue'
import ServerPing from './ServerPing.vue'
import ServerRecentPlays from './ServerRecentPlays.vue'
import ServerRegion from './ServerRegion.vue'
import ServerRequiredContent from './ServerRequiredContent.vue'

defineProps<{
	onlinePlayers?: number
	recentPlays?: number
	region?: string
	ping?: number
	requiredContent?: {
		name: string
		icon?: string
	}
}>()
</script>
<template>
	<div class="empty:hidden flex items-center gap-2">
		<ServerOnlinePlayers v-if="onlinePlayers !== undefined" :online="onlinePlayers" />
		<ServerRecentPlays v-if="recentPlays !== undefined" :recent-plays="recentPlays" />
		<ServerRegion v-if="region" :region="region" />
		<ServerPing v-if="ping" :ping="ping" />
		<ServerRequiredContent
			v-if="requiredContent"
			:name="requiredContent.name"
			:icon="requiredContent.icon"
		/>
	</div>
</template>
