<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'

import { defineMessage, useVIntl } from '../../composables/i18n'

defineProps<{
	href: string
}>()

const { formatMessage } = useVIntl()

const label = defineMessage({
	id: 'external-files.permissions-card.original-project-page',
	defaultMessage: 'Original project',
})
</script>
<template>
	<a
		class="text-link flex items-center outline-offset-[4px] rounded-sm truncate"
		target="_blank"
		rel="noopener"
		:href="href"
	>
		<span class="truncate">{{ formatMessage(label) }}</span>
		<ExternalIcon class="size-3 shrink-0 mb-2 ml-1" />
	</a>
</template>
