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

		<div v-if="minecraftServer?.region || ping" class="w-1.5 h-1.5 rounded-full bg-surface-5"></div>

		<div v-if="linkedProjectV3" class="flex gap-1.5 items-center font-medium text-primary">
			Linked to
			<Avatar
				:src="linkedProjectV3.icon_url"
				:alt="linkedProjectV3.name"
				:tint-by="instanceId"
				size="24px"
			/>
			<router-link
				:to="`/project/${linkedProjectV3.slug ?? linkedProjectV3.id}`"
				class="hover:underline text-primary truncate"
			>
				{{ linkedProjectV3.name }}
			</router-link>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	Avatar,
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
	linkedProjectV3?: Labrinth.Projects.v3.Project
	instanceId?: string
}>()
</script>
