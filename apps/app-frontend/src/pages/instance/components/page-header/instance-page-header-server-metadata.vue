<template>
	<PageHeaderMetadata>
		<PageHeaderMetadataItem v-if="loadingServerPing && playersOnline !== undefined">
			<ServerOnlinePlayers :online="playersOnline" :status-online="statusOnline" hide-label />
		</PageHeaderMetadataItem>
		<PageHeaderMetadataItem
			v-if="minecraftServer?.region || loadingServerPing || hostingStatus !== undefined"
		>
			<ServerRegion v-if="minecraftServer?.region" :region="minecraftServer.region" flag-only />
			<template v-if="hostingStatus !== undefined">
				<TagItem
					class="border !border-solid !font-medium w-max"
					:class="{
						'border-green bg-highlight-green text-green': hostingStatus === 'running',
						'border-red bg-highlight-red text-red': hostingStatus === 'stopped',
					}"
				>
					<SignalIcon />
					{{
						formatMessage(
							hostingStatus === 'running'
								? messages.online
								: hostingStatus === 'stopped'
									? messages.offline
									: messages.unknown,
						)
					}}
				</TagItem>
				<ServerPing v-if="ping !== undefined" :ping="ping" />
			</template>
			<ServerPing v-else-if="loadingServerPing" :ping="ping" :status-online="statusOnline" />
			<SpinnerIcon
				v-else
				class="size-4 animate-spin"
				:aria-label="formatMessage(commonMessages.loadingLabel)"
				role="status"
			/>
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
import type { Labrinth, SharedInstances } from '@modrinth/api-client'
import { SignalIcon, SpinnerIcon, TimerIcon } from '@modrinth/assets'
import {
	commonMessages,
	defineMessages,
	PageHeaderMetadata,
	PageHeaderMetadataItem,
	ServerOnlinePlayers,
	ServerPing,
	ServerRegion,
	TagItem,
	useVIntl,
} from '@modrinth/ui'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	online: { id: 'project.server.status.online', defaultMessage: 'Online' },
	offline: { id: 'project.server.status.offline', defaultMessage: 'Offline' },
	unknown: { id: 'app.instance.server.status.unknown', defaultMessage: 'Status unknown' },
})

defineProps<{
	loadingServerPing?: boolean
	playersOnline?: number
	hostingStatus?: SharedInstances.Instances.v1.OnlineStatus
	statusOnline?: boolean
	ping?: number
	minecraftServer?: Labrinth.Projects.v3.Project['minecraft_server']
	showInstancePlayTime?: boolean
	playtimeLabel?: string
}>()
</script>
