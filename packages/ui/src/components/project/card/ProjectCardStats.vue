<script setup lang="ts">
import { DownloadIcon, HeartIcon } from '@modrinth/assets'

import { capitalizeString } from '../../../../../utils'
import { useCompactNumber, useVIntl } from '../../../composables'
import { commonMessages } from '../../../utils'

const { formatMessage } = useVIntl()
const { formatCompactNumber } = useCompactNumber()

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
					count: downloads,
				}),
			)
		"
		class="flex items-center gap-2 trim-text-box smart-clickable:allow-pointer-events"
	>
		<DownloadIcon class="size-5 shrink-0" />
		<span class="font-medium">
			{{ formatCompactNumber(downloads) }}
		</span>
	</div>
	<div
		v-if="followers !== undefined"
		v-tooltip="
			capitalizeString(
				formatMessage(commonMessages.projectFollowers, {
					count: followers,
				}),
			)
		"
		class="flex items-center gap-2 trim-text-box smart-clickable:allow-pointer-events"
	>
		<HeartIcon class="size-5 shrink-0" />
		<span class="font-medium">
			{{ formatCompactNumber(followers) }}
		</span>
	</div>
</template>
<style scoped>
.trim-text-box {
	text-box-trim: trim-both;
}
</style>
