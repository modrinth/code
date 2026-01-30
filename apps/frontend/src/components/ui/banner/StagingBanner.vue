<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	PagewideBanner,
	useVIntl,
} from '@modrinth/ui'

const { formatMessage } = useVIntl()
const cosmetics = useCosmetics()

const messages = defineMessages({
	title: {
		id: 'layout.banner.staging.title',
		defaultMessage: 'You’re viewing Modrinth’s staging environment',
	},
	description: {
		id: 'layout.banner.staging.description',
		defaultMessage:
			'The staging environment is completely separate from the production Modrinth database. This is used for testing and debugging purposes, and may be running in-development versions of the Modrinth backend or frontend newer than the production instance.',
	},
})

function hideStagingBanner() {
	cosmetics.value.hideStagingBanner = true
}
</script>

<template>
	<PagewideBanner v-if="!cosmetics.hideStagingBanner" variant="warning">
		<template #title>
			<span>{{ formatMessage(messages.title) }}</span>
		</template>
		<template #description>
			{{ formatMessage(messages.description) }}
		</template>
		<template #actions_right>
			<ButtonStyled type="transparent" circular>
				<button :aria-label="formatMessage(commonMessages.closeButton)" @click="hideStagingBanner">
					<XIcon aria-hidden="true" />
				</button>
			</ButtonStyled>
		</template>
	</PagewideBanner>
</template>
