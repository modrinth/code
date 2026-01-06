<script setup lang="ts">
import { defineMessages, PagewideBanner, useVIntl } from '@modrinth/ui'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'layout.banner.build-fail.title',
		defaultMessage: 'Error generating state from API when building.',
	},
	description: {
		id: 'layout.banner.build-fail.description',
		defaultMessage:
			"This deploy of Modrinth's frontend failed to generate state from the API. This may be due to an outage or an error in configuration. Rebuild when the API is available. Error codes: {errors}; Current API URL is: {url}",
	},
})

defineProps<{
	errors: any[] | undefined
	apiUrl: string
}>()
</script>

<template>
	<PagewideBanner v-if="errors?.length" variant="error">
		<template #title>
			<span>{{ formatMessage(messages.title) }}</span>
		</template>
		<template #description>
			{{
				formatMessage(messages.description, {
					errors: JSON.stringify(errors),
					url: apiUrl,
				})
			}}
		</template>
	</PagewideBanner>
</template>
