<template>
	<PageHeaderMetadataItem
		:icon="TagIcon"
		:icon-props="{ tag: loaderDisplayName, enforceType: 'loader' }"
		tooltip="Mod loader and Minecraft version"
	>
		{{ loaderLabel }}
	</PageHeaderMetadataItem>
	<PageHeaderMetadataItem
		v-if="showInstancePlayTime && playtimeLabel"
		:icon="TimerIcon"
		tooltip="Total playtime"
	>
		{{ playtimeLabel }}
	</PageHeaderMetadataItem>
	<PageHeaderMetadataTimeItem
		v-if="instance.last_played"
		:icon="ClockIcon"
		:date="instance.last_played"
		:label="formatMessage(playtimeMessages.lastPlayed)"
	/>
	<PageHeaderMetadataItem v-else :icon="ClockIcon" tooltip="Last played">
		{{ formatMessage(playtimeMessages.neverPlayed) }}
	</PageHeaderMetadataItem>
</template>

<script setup lang="ts">
import { ClockIcon, TimerIcon } from '@modrinth/assets'
import {
	formatLoaderLabel,
	PageHeaderMetadataItem,
	PageHeaderMetadataTimeItem,
	type ServerLoader,
	TagIcon,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import type { GameInstance } from '@/helpers/types'

import { formatPlaytime, playtimeMessages } from './playtime'

const props = defineProps<{
	instance: GameInstance
	showInstancePlayTime?: boolean
}>()

const { formatMessage } = useVIntl()

const loaderDisplayName = computed(() => formatLoaderLabel(props.instance.loader) as ServerLoader)
const loaderLabel = computed(() =>
	[loaderDisplayName.value, props.instance.game_version].filter(Boolean).join(' '),
)
const playtimeLabel = computed(() =>
	formatPlaytime(props.instance.recent_time_played + props.instance.submitted_time_played),
)
</script>
