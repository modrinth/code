<template>
	<div class="flex w-full flex-col gap-1">
		<span class="font-semibold text-contrast">{{ formatMessage(messages.title) }}</span>
		<p class="m-0 text-primary">{{ formatMessage(messages.description) }}</p>
		<a
			:href="learnMoreUrl"
			target="_blank"
			rel="noopener noreferrer"
			class="flex w-fit items-center gap-1.5 font-medium text-link hover:underline"
		>
			{{ formatMessage(messages.learnMore) }}
			<ExternalIcon class="size-4" aria-hidden="true" />
		</a>
	</div>
</template>

<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'

import { defineMessages, useVIntl } from '../../composables/i18n'

// TODO: link to the sandbox article when ready
const learnMoreUrl = 'https://modrinth.com/news'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'unknown-file-sandbox-notice.title',
		defaultMessage: 'Limit mod access with a sandbox',
	},
	description: {
		id: 'unknown-file-sandbox-notice.description',
		defaultMessage:
			'The sandbox isolates Minecraft, limiting mods to your system files, .minecraft, and anything we allow or you allow below. Some mods may not work if the sandbox blocks access they need.',
	},
	learnMore: {
		id: 'unknown-file-sandbox-notice.learn-more',
		defaultMessage: 'Learn more',
	},
})
</script>
