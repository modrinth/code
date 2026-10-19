<template>
	<LauncherOpening
		v-if="launcherHandoff?.deeplinkUrl"
		:localhost-url="launcherHandoff.localhostUrl ?? undefined"
		:deeplink-url="launcherHandoff.deeplinkUrl"
	/>
	<div v-else-if="launcherHandoff?.localhostUrl">
		<iframe
			:src="launcherHandoff.localhostUrl"
			class="fixed left-0 top-0 z-[9999] m-0 h-full w-full border-0 p-0"
		></iframe>
	</div>
	<CreateAccountView
		v-else
		v-model:date-of-birth="dateOfBirth"
		v-model:username="username"
		v-model:token="token"
		v-model:subscribe="subscribe"
		:globals="globals"
		:requires-dob="requiresDob"
		:on-complete-sign-up="completeOAuthSignUp"
		:on-set-captcha-ref="setCaptchaRef"
	/>
</template>

<script setup lang="ts">
import {
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'

import CreateAccountView from '@/components/ui/auth/CreateAccount.vue'
import LauncherOpening from '@/components/ui/auth/LauncherOpening.vue'
import { rememberStoredAccount } from '@/composables/accounts.ts'
import { promotePendingSignInOAuthProvider } from '@/composables/auth.ts'
import {
	createLauncherHandoff,
	getQueryString,
	hideLauncherSessionCode,
	isLauncherProtocolV2,
	launcherAuthMessages,
	type LauncherHandoff,
} from '@/composables/launcher-auth.ts'

interface AuthGlobalsResponse {
	captcha_enabled?: boolean
	[key: string]: unknown
}

interface ApiErrorShape {
	data?: {
		description?: string
	}
}

const getErrorMessage = (error: unknown): string => {
	const apiError = error as ApiErrorShape
	if (typeof apiError?.data?.description === 'string') {
		return apiError.data.description
	}
	if (error instanceof Error) {
		return error.message
	}
	return String(error)
}

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const route = useNativeRoute()
const auth = await useAuth()

const messages = defineMessages({
	createAccountTitle: {
		id: 'auth.create-account.page-title',
		defaultMessage: 'Create Account',
	},
})

useHead({
	title() {
		return `${formatMessage(messages.createAccountTitle)} - Modrinth`
	},
})

const requiresDob = computed(() => {
	const raw = route.query.requires_dob
	const value = Array.isArray(raw) ? raw[0] : raw

	if (!value) {
		return false
	}

	return value === 'true' || value === '1'
})

const oauthFlowState = computed(() => {
	const state = route.query.state
	const value = Array.isArray(state) ? state[0] : state
	return typeof value === 'string' ? value : ''
})

const defaultUsername = computed(() => {
	const queryUsername = route.query.username
	const value = Array.isArray(queryUsername) ? queryUsername[0] : queryUsername
	return typeof value === 'string' && value.length > 0 ? value : ''
})

const dateOfBirth = ref('')
const username = ref(defaultUsername.value)
const token = ref('')
const subscribe = ref(false)
type LauncherCallback = Extract<LauncherHandoff, { type: 'callback' }>
const launcherHandoff = ref<LauncherCallback | null>(null)
const isProtocolV2 = isLauncherProtocolV2(route)

const captcha = ref<{ reset?: () => void } | null>(null)
const setCaptchaRef = (captchaRef: unknown) => {
	captcha.value = (captchaRef as { reset?: () => void } | null) ?? null
}

const { data: globals } = useQuery<AuthGlobalsResponse>({
	queryKey: ['auth-globals'],
	queryFn: async () => {
		try {
			return await client.labrinth.globals_internal.get()
		} catch (err) {
			console.error('Error fetching globals:', err)
			return { captcha_enabled: true, tax_compliance_thresholds: {} }
		}
	},
})

async function completeOAuthSignUp(accountConsent: boolean) {
	startLoading()
	try {
		if (!oauthFlowState.value) {
			throw new Error('Missing OAuth flow state')
		}

		const res = await client.labrinth.auth_v2.createOAuthAccount({
			username: username.value.trim() || defaultUsername.value,
			state: oauthFlowState.value,
			challenge: token.value,
			sign_up_newsletter: subscribe.value,
			account_consent: accountConsent,
			app_session: isProtocolV2,
		})

		await finishSignIn(res.session, res.app_session)
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: getErrorMessage(err),
			type: 'error',
		})
		captcha.value?.reset?.()
	}
	stopLoading()
}

async function finishSignIn(sessionToken?: string | null, appSessionToken?: string | null) {
	if (route.query.launcher) {
		if (isProtocolV2) {
			if (!sessionToken) return

			try {
				if (!appSessionToken) {
					throw new Error(formatMessage(launcherAuthMessages.handoffFailed))
				}

				await useAuth(sessionToken)
				await useUser()
				queryClient.clear()
				const signedIn = await useAuth()
				if (signedIn.value.user && signedIn.value.token) {
					rememberStoredAccount(signedIn.value.user, signedIn.value.token)
				}
				promotePendingSignInOAuthProvider()

				const handoff = await createLauncherHandoff(route, appSessionToken)
				if (handoff.type === 'external') {
					await navigateTo(handoff.url, {
						external: true,
					})
					return
				}

				launcherHandoff.value = handoff
				hideLauncherSessionCode()
			} catch (err) {
				console.error(err)
				addNotification({
					title: formatMessage(commonMessages.errorNotificationTitle),
					text: getErrorMessage(err),
					type: 'error',
				})
			}

			return
		}

		const token = sessionToken ?? auth.value.token
		if (!token) return

		try {
			const handoff = await createLauncherHandoff(route, token)
			if (handoff.type === 'external') {
				promotePendingSignInOAuthProvider()
				await navigateTo(handoff.url, {
					external: true,
				})
				return
			}

			promotePendingSignInOAuthProvider()
			launcherHandoff.value = handoff
		} catch (err) {
			console.error(err)
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: formatMessage(launcherAuthMessages.handoffFailed),
				type: 'error',
			})
		}

		return
	}

	if (sessionToken) {
		await useAuth(sessionToken)
		await useUser()
		queryClient.clear()

		promotePendingSignInOAuthProvider()
	}

	if (route.query.redirect) {
		const redirect = decodeURIComponent(getQueryString(route.query.redirect) ?? '')
		await navigateTo(redirect, {
			replace: true,
		})
	} else if (auth.value.user) {
		await navigateTo(`/user/${auth.value.user.username}`)
	}
}
</script>
