<script setup lang="ts">
import { SettingsIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, PagewideBanner, useVIntl } from '@modrinth/ui'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'layout.banner.subscription-payment-failed.title',
		defaultMessage: 'Billing action required.',
	},
	description: {
		id: 'layout.banner.subscription-payment-failed.description',
		defaultMessage:
			'One or more subscriptions failed to renew. Please update your payment method to prevent losing access!',
	},
	action: {
		id: 'layout.banner.subscription-payment-failed.button',
		defaultMessage: 'Update billing info',
	},
})
</script>

<template>
	<PagewideBanner variant="error">
		<template #title>
			<span>{{ formatMessage(messages.title) }}</span>
		</template>
		<template #description>
			<span>{{ formatMessage(messages.description) }}</span>
		</template>
		<template #actions_right>
			<ButtonStyled color="red">
				<nuxt-link to="/settings/billing">
					<SettingsIcon aria-hidden="true" />
					{{ formatMessage(messages.action) }}
				</nuxt-link>
			</ButtonStyled>
		</template>
	</PagewideBanner>
</template>
