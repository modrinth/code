<template>
	<AuthCard>
		<template v-if="auth.user && auth.user.email_verified && !success">
			<h1 class="m-0 mx-auto text-xl font-semibold text-contrast">
				{{ formatMessage(alreadyVerifiedMessages.title) }}
			</h1>

			<Admonition type="success">
				{{ formatMessage(alreadyVerifiedMessages.description) }}
			</Admonition>

			<div class="grid grid-cols-2 gap-2">
				<ButtonLink class="shadow-none" to="/settings/account">
					<SettingsIcon /> {{ formatMessage(messages.accountSettings) }}
				</ButtonLink>
				<ButtonLink type="colored" color="brand" to="/discover/mods">
					{{ formatMessage(messages.discoverMods) }}
					<RightArrowIcon />
				</ButtonLink>
			</div>
		</template>

		<template v-else-if="success">
			<h1 class="m-0 mx-auto text-xl font-semibold text-contrast">
				{{ formatMessage(postVerificationMessages.title) }}
			</h1>

			<Admonition type="success">
				{{ formatMessage(postVerificationMessages.description) }}
			</Admonition>

			<template v-if="auth.user">
				<div class="grid grid-cols-2 gap-2">
					<ButtonLink to="/settings/account">
						<SettingsIcon /> {{ formatMessage(messages.accountSettings) }}
					</ButtonLink>
					<ButtonLink type="colored" color="brand" to="/discover/mods">
						{{ formatMessage(messages.discoverMods) }}
						<RightArrowIcon />
					</ButtonLink>
				</div>
			</template>
			<ButtonLink v-else type="colored" color="brand" to="/auth/sign-in" class="!w-full">
				{{ formatMessage(commonMessages.signInButton) }}
				<RightArrowIcon />
			</ButtonLink>
		</template>

		<template v-else>
			<h1 class="m-0 mx-auto text-xl font-semibold text-contrast">
				{{ formatMessage(failedVerificationMessages.title) }}
			</h1>

			<Admonition v-if="auth.user" type="warning">
				{{ formatMessage(failedVerificationMessages.loggedInDescription) }}
			</Admonition>
			<Admonition v-else type="warning">
				{{ formatMessage(failedVerificationMessages.description) }}
			</Admonition>

			<Button
				v-if="auth.user"
				type="colored"
				color="brand"
				class="!w-full"
				@click="handleResendEmailVerification"
			>
				{{ formatMessage(failedVerificationMessages.action) }}
				<RightArrowIcon />
			</Button>
			<ButtonLink v-else type="colored" color="brand" to="/auth/sign-in" class="!w-full">
				{{ formatMessage(commonMessages.signInButton) }}
				<RightArrowIcon />
			</ButtonLink>
		</template>
	</AuthCard>
</template>
<script setup lang="ts">
import { RightArrowIcon, SettingsIcon } from '@modrinth/assets'
import {
	Admonition,
	Button,
	ButtonLink,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'

import AuthCard from '@/components/ui/auth/AuthCard.vue'
import { authQueryString, getAuthErrorMessage } from '@/composables/auth-form.ts'
import { useAuthPageTitle, useAuthRoute } from '@/composables/auth-modal.ts'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'auth.verify-email.title',
		defaultMessage: 'Verify Email',
	},
	accountSettings: {
		id: 'auth.verify-email.action.account-settings',
		defaultMessage: 'Account settings',
	},
	discoverMods: {
		id: 'auth.verify-email.action.discover-mods',
		defaultMessage: 'Discover mods',
	},
	emailSentNotificationTitle: {
		id: 'auth.verify-email.notification.email-sent.title',
		defaultMessage: 'Email sent',
	},
	emailSentNotificationDescription: {
		id: 'auth.verify-email.notification.email-sent.description',
		defaultMessage: 'An email with a link to verify your account has been sent to {email}.',
	},
	errorOccurredTitle: {
		id: 'auth.verify-email.notification.error-occurred.title',
		defaultMessage: 'An error occurred',
	},
})

const alreadyVerifiedMessages = defineMessages({
	title: {
		id: 'auth.verify-email.already-verified.title',
		defaultMessage: 'Email already verified',
	},
	description: {
		id: 'auth.verify-email.already-verified.description',
		defaultMessage: 'Your email is already verified!',
	},
})

const postVerificationMessages = defineMessages({
	title: {
		id: 'auth.verify-email.post-verification.title',
		defaultMessage: 'Email verification',
	},
	description: {
		id: 'auth.verify-email.post-verification.description',
		defaultMessage: 'Your email address has been successfully verified!',
	},
})

const failedVerificationMessages = defineMessages({
	title: {
		id: 'auth.verify-email.failed-verification.title',
		defaultMessage: 'Email verification failed',
	},
	description: {
		id: 'auth.verify-email.failed-verification.description',
		defaultMessage:
			'We were unable to verify your email. Try re-sending the verification email through your dashboard by signing in.',
	},
	loggedInDescription: {
		id: 'auth.verify-email.failed-verification.description.logged-in',
		defaultMessage:
			'We were unable to verify your email. Try re-sending the verification email through the button below.',
	},
	action: {
		id: 'auth.verify-email.failed-verification.action',
		defaultMessage: 'Resend verification email',
	},
})

useAuthPageTitle(() => formatMessage(messages.title))

const route = useAuthRoute()
const auth = await useAuth()

const success = ref(false)

if (route.query.flow) {
	try {
		const emailVerified = useState<boolean | null>('emailVerified', () => null)

		if (emailVerified.value === null) {
			await useBaseFetch('auth/email/verify', {
				method: 'POST',
				body: {
					flow: authQueryString(route.query.flow),
				},
			})
			emailVerified.value = true
			success.value = true
		}

		if (emailVerified.value) {
			success.value = true

			if (auth.value.token) {
				await useAuth(auth.value.token)
			}
		}
	} catch {
		success.value = false
	}
}

async function handleResendEmailVerification() {
	try {
		await resendVerifyEmail()
		addNotification({
			title: formatMessage(messages.emailSentNotificationTitle),
			text: formatMessage(messages.emailSentNotificationDescription, {
				email: auth.value.user?.email ?? '',
			}),
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorOccurredTitle),
			text: getAuthErrorMessage(err),
			type: 'error',
		})
	}
}
</script>
