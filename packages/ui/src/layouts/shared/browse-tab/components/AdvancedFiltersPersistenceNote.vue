<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	persistenceNote: {
		id: 'browse.advanced-filters.persistence-note',
		defaultMessage:
			'Advanced exclusions are saved across sessions. Opening a shared link with filters will override them.',
	},
})
</script>

<template>
	<div class="mb-2 flex gap-2 px-3 text-sm text-secondary opacity-80">
		<InfoIcon class="mt-0.5 size-4 shrink-0" />
		<span>{{ formatMessage(messages.persistenceNote) }}</span>
	</div>
</template>
