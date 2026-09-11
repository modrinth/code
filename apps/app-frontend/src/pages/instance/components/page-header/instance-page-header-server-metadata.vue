<template>
	<PageHeaderMetadata>
		<PageHeaderMetadataItem v-if="loadingServerPing && playersOnline !== undefined">
			<ServerOnlinePlayers :online="playersOnline" :status-online="statusOnline" hide-label />
		</PageHeaderMetadataItem>
		<PageHeaderMetadataItem
			v-if="minecraftServer?.region || loadingServerPing"
		>
			<ServerRegion v-if="minecraftServer?.region" :region="minecraftServer.region" flag-only />
			<ServerPing
				v-if="loadingServerPing"
				:ping="ping"
				:status-online="statusOnline"
			/>
			<SpinnerIcon v-else class="size-4 animate-spin" :aria-label="formatMessage(commonMessages.loadingLabel)" role="status" />
		</PageHeaderMetadataItem>
		<PageHeaderMetadataItem
			v-if="showInstancePlayTime && playtimeLabel"
			:icon="TimerIcon"
			tooltip="Total playtime"
		>
			{{ playtimeLabel }}
		</PageHeaderMetadataItem>
	</PageHeaderMetadata>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { SpinnerIcon, TimerIcon } from '@modrinth/assets'
import {
	commonMessages,
	PageHeaderMetadata,
	PageHeaderMetadataItem,
	ServerOnlinePlayers,
	ServerPing,
	ServerRegion,
	useVIntl,
} from '@modrinth/ui'

const { formatMessage } = useVIntl()

defineProps<{
	loadingServerPing?: boolean
	playersOnline?: number
	statusOnline?: boolean
	ping?: number
	minecraftServer?: Labrinth.Projects.v3.Project['minecraft_server']
	showInstancePlayTime?: boolean
	playtimeLabel?: string
}>()
</script>
