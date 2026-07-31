<script setup lang="ts">
import { XCircleIcon, XIcon } from '@modrinth/assets'
import { defineMessages, PagewideBanner, useVIntl } from '@modrinth/ui'
import Button from '@modrinth/ui/src/components/base/buttons/Button.vue'

const { formatMessage } = useVIntl()
const flags = useFeatureFlags()

const tempIgnored = ref(false)

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
	ignoreErrors: {
		id: 'layout.banner.build-fail.ignore',
		defaultMessage: 'Ignore',
	},
	alwaysIgnore: {
		id: 'layout.banner.build-fail.always-ignore',
		defaultMessage: 'Always ignore',
	},
})

defineProps<{
	errors: any[] | undefined
	apiUrl: string
}>()

function alwaysIgnoreBanner() {
	flags.value.alwaysIgnoreErrorBanner = true
	saveFeatureFlags()
}
</script>

<template>
	<PagewideBanner
		v-if="
			flags.showAllBanners || (errors?.length && !tempIgnored && !flags.alwaysIgnoreErrorBanner)
		"
		variant="error"
	>
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
		<template #actions_right>
			<Button type="quiet" color="red" @click="alwaysIgnoreBanner">
				<XCircleIcon aria-hidden="true" />
				{{ formatMessage(messages.alwaysIgnore) }}
			</Button>
			<Button type="colored" color="red" @click="tempIgnored = true">
				<XIcon aria-hidden="true" />
				{{ formatMessage(messages.ignoreErrors) }}
			</Button>
		</template>
	</PagewideBanner>
</template>
