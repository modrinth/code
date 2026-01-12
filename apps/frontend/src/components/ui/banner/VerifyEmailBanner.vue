<script setup lang="ts">
import { SettingsIcon } from '@modrinth/assets'
import { defineMessages, injectNotificationManager, PagewideBanner, useVIntl } from '@modrinth/ui'
import { FetchError } from 'ofetch'

const { addNotification } = injectNotificationManager()

const { formatMessage } = useVIntl()

defineProps<{
	hasEmail: boolean
}>()

const verifyEmailBannerMessages = defineMessages({
	title: {
		id: 'layout.banner.account-action',
		defaultMessage: 'Account action required',
	},
	description: {
		id: 'layout.banner.verify-email.description',
		defaultMessage:
			'For security reasons, Modrinth needs you to verify the email address associated with your account.',
	},
	action: {
		id: 'layout.banner.verify-email.action',
		defaultMessage: 'Re-send verification email',
	},
})

const addEmailBannerMessages = defineMessages({
	title: {
		id: 'layout.banner.account-action',
		defaultMessage: 'Account action required',
	},
	description: {
		id: 'layout.banner.add-email.description',
		defaultMessage:
			'For security reasons, Modrinth needs you to register an email address to your account.',
	},
	action: {
		id: 'layout.banner.add-email.button',
		defaultMessage: 'Visit account settings',
	},
})

async function handleResendEmailVerification() {
	try {
		await resendVerifyEmail()
		addNotification({
			title: 'Verification email sent',
			text: 'Please check your inbox for the verification email.',
			type: 'success',
		})
	} catch (err) {
		if (err instanceof FetchError) {
			const description = err.data?.description || err.message
			addNotification({
				title: 'An error occurred',
				text: description,
				type: 'error',
			})
		} else {
			addNotification({
				title: 'An error occurred',
				text: `${err}`,
				type: 'error',
			})
		}
	}
}
</script>

<template>
	<PagewideBanner variant="warning">
		<template #title>
			<span>
				{{
					hasEmail
						? formatMessage(verifyEmailBannerMessages.title)
						: formatMessage(addEmailBannerMessages.title)
				}}
			</span>
		</template>
		<template #description>
			<span>
				{{
					hasEmail
						? formatMessage(verifyEmailBannerMessages.description)
						: formatMessage(addEmailBannerMessages.description)
				}}
			</span>
		</template>
		<template #actions>
			<button v-if="hasEmail" class="btn" @click="handleResendEmailVerification">
				{{ formatMessage(verifyEmailBannerMessages.action) }}
			</button>
			<nuxt-link v-else class="btn" to="/settings/account">
				<SettingsIcon aria-hidden="true" />
				{{ formatMessage(addEmailBannerMessages.action) }}
			</nuxt-link>
		</template>
	</PagewideBanner>
</template>
