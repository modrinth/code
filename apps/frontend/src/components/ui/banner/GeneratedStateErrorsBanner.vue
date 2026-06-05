<script setup lang="ts">
import { XCircleIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, PagewideBanner, useVIntl } from '@modrinth/ui'

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
			<ButtonStyled color="red" type="transparent" hover-color-fill="background">
				<button @click="alwaysIgnoreBanner">
					<XCircleIcon />
					{{ formatMessage(messages.alwaysIgnore) }}
				</button>
			</ButtonStyled>
			<ButtonStyled color="red">
				<button @click="tempIgnored = true">
					<XIcon />
					{{ formatMessage(messages.ignoreErrors) }}
				</button>
			</ButtonStyled>
		</template>
	</PagewideBanner>
</template>
