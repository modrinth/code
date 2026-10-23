<template>
	<div
		v-if="stage === 'reading' || (current && (!fullyLoaded || fetching))"
		class="flex items-center gap-1.5 text-xs text-secondary"
		role="status"
	>
		<SpinnerIcon class="size-3.5 animate-spin" aria-hidden="true" />
		{{ formatMessage(label) }}
		<span v-if="fetchedLabel" class="tabular-nums">({{ fetchedLabel }})</span>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { useFormatBytes } from '#ui/composables/format-bytes'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectArchiveSession } from '../providers/archive-session'

/** What's still being fetched or read of the session's archive, for a toolbar. */
const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()
const { stage, progress, fullyLoaded, current } = injectArchiveSession()

const messages = defineMessages({
	reading: {
		id: 'archive-viewer.reading-index',
		defaultMessage: 'Reading the archive...',
	},
	readingNested: {
		id: 'archive-viewer.reading-nested',
		defaultMessage: 'Reading nested archives...',
	},
	fetchingRest: {
		id: 'archive-viewer.fetching-rest',
		defaultMessage: 'Fetching the rest of the archive...',
	},
	fetchedProgress: {
		id: 'archive-viewer.fetched-progress',
		defaultMessage: '{loaded} of {total} fetched',
	},
})

/** Whether the rest of the archive is still being fetched in the background. */
const fetching = computed(() => {
	const { loaded, total } = progress.value
	return total != null && loaded < total
})

const label = computed(() => {
	if (stage.value === 'reading') return messages.reading
	return fullyLoaded.value ? messages.fetchingRest : messages.readingNested
})

const fetchedLabel = computed(() => {
	const { loaded, total } = progress.value
	if (!loaded || !total) return null
	return formatMessage(messages.fetchedProgress, {
		loaded: formatBytes(loaded),
		total: formatBytes(total),
	})
})
</script>
