<script setup lang="ts">
import { capitalizeString } from '@modrinth/utils'

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
		<span class="font-medium"> {{ formatCompactNumber(downloads) }} downloads </span>
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
		<span class="font-medium"> {{ formatCompactNumber(followers) }} followers </span>
	</div>
</template>
<style scoped>
.trim-text-box {
	text-box-trim: trim-both;
}
</style>
