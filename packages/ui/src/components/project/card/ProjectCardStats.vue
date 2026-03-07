<script setup lang="ts">
import { DownloadIcon, HeartIcon } from '@modrinth/assets'
import { capitalizeString, formatNumber } from '@modrinth/utils'

import { useVIntl } from '../../../composables'
import { commonMessages } from '../../../utils'

const { formatMessage } = useVIntl()

defineProps<{
	downloads?: number
	followers?: number
}>()
</script>

<template>
	<div
		v-if="downloads !== undefined"
		v-tooltip="
			capitalizeString(
				formatMessage(commonMessages.projectDownloads, {
					count: formatNumber(downloads, false),
				}),
			)
		"
		class="flex items-center gap-2 trim-text-box smart-clickable:allow-pointer-events"
	>
		<DownloadIcon class="size-5 shrink-0" />
		<span class="font-medium">
			{{ formatNumber(downloads) }}
		</span>
	</div>
	<div
		v-if="followers !== undefined"
		v-tooltip="
			capitalizeString(
				formatMessage(commonMessages.projectFollowers, {
					count: formatNumber(followers, false),
				}),
			)
		"
		class="flex items-center gap-2 trim-text-box smart-clickable:allow-pointer-events"
	>
		<HeartIcon class="size-5 shrink-0" />
		<span class="font-medium">
			{{ formatNumber(followers) }}
		</span>
	</div>
</template>
<style scoped>
.trim-text-box {
	text-box-trim: trim-both;
}
</style>
