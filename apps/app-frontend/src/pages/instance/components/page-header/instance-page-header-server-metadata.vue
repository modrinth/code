<template>
	<div class="flex items-center flex-wrap gap-2">
		<template v-if="loadingServerPing">
			<ServerOnlinePlayers
				v-if="playersOnline !== undefined"
				:online="playersOnline"
				:status-online="statusOnline"
				hide-label
			/>
			<ServerRecentPlays :recent-plays="recentPlays ?? 0" hide-label />
			<div
				v-if="
					(playersOnline !== undefined || recentPlays !== undefined) &&
					(minecraftServer?.region || ping)
				"
				class="w-1.5 h-1.5 rounded-full bg-surface-5"
			></div>
			<ServerPing v-if="ping" :ping="ping" />
		</template>

		<ServerRegion v-if="minecraftServer?.region" :region="minecraftServer?.region" />

		<div
			v-if="showInstancePlayTime && playtimeLabel && (loadingServerPing || minecraftServer?.region || ping)"
			class="h-1.5 w-1.5 rounded-full bg-surface-5"
		></div>

		<div
			v-if="showInstancePlayTime && playtimeLabel"
			v-tooltip="'Total playtime'"
			class="flex items-center gap-1.5 font-medium text-secondary"
		>
			<TimerIcon aria-hidden="true" class="size-5 shrink-0 text-current" />
			{{ playtimeLabel }}
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { TimerIcon } from '@modrinth/assets'
import {
	ServerOnlinePlayers,
	ServerPing,
	ServerRecentPlays,
	ServerRegion,
} from '@modrinth/ui'

defineProps<{
	loadingServerPing?: boolean
	playersOnline?: number
	statusOnline?: boolean
	recentPlays?: number
	ping?: number
	minecraftServer?: Labrinth.Projects.v3.Project['minecraft_server']
	showInstancePlayTime?: boolean
	playtimeLabel?: string
}>()
</script>
