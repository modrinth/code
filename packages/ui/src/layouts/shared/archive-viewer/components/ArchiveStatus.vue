<template>
	<Admonition
		v-if="stage === 'error'"
		type="critical"
		:header="formatMessage(messages.errorTitle, { name: source?.name ?? '' })"
		:body="error?.message"
	>
		<template #actions>
			<Button @click="retry">
				<RefreshCwIcon aria-hidden="true" />
				{{ formatMessage(commonMessages.retryButton) }}
			</Button>
		</template>
	</Admonition>
	<Admonition
		v-else-if="stage === 'reading'"
		type="info"
		:header="formatMessage(messages.reading, { name: source?.name ?? '' })"
		:body="formatMessage(messages.readingDescription)"
		waiting
	/>
	<slot v-else name="empty" />
</template>

<script setup lang="ts">
import { RefreshCwIcon } from '@modrinth/assets'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { injectArchiveSession } from '../providers/archive-session'

/** How opening the session's archive is going: being read, failed, or nothing to open. */
defineSlots<{
	/** Shown while there's no archive to open. */
	empty?: () => unknown
}>()

const { formatMessage } = useVIntl()
const { stage, error, source, retry } = injectArchiveSession()

const messages = defineMessages({
	reading: {
		id: 'archive-viewer.reading',
		defaultMessage: 'Reading {name}',
	},
	readingDescription: {
		id: 'archive-viewer.reading-description',
		defaultMessage: "Fetching the archive's index. Files are fetched as they are opened.",
	},
	errorTitle: {
		id: 'archive-viewer.error-title',
		defaultMessage: 'Unable to open {name}',
	},
})
</script>
