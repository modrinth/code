<template>
	<SignInView
		v-model:email="email"
		v-model:password="password"
		v-model:token="token"
		v-model:two-factor-code="twoFactorCode"
		:subtle-launcher-redirect-uri="subtleLauncherRedirectUri"
		:flow="flow"
		:redirect-target="redirectTarget"
		:route-query="route.query"
		:globals="globals"
		:on-password-sign-in="beginPasswordSignIn"
		:on-two-factor-sign-in="begin2FASignIn"
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
import { useStorage } from '@vueuse/core'
import type { LocationQueryValue } from 'vue-router'

import SignInView from '@/components/ui/auth/SignIn.vue'
import {
	getLauncherRedirectUrl,
	LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	promotePendingSignInOAuthProvider,
} from '@/composables/auth.ts'

type AuthProvider = 'discord' | 'google' | 'github' | 'gitlab' | 'steam' | 'microsoft'

interface AuthGlobalsResponse {
	captcha_enabled?: boolean
	[key: string]: unknown
}

interface ApiErrorShape {
	data?: {
		description?: string
	}
}

const getQueryString = (
	value: LocationQueryValue | LocationQueryValue[] | null | undefined,
): string => {
	const firstValue = Array.isArray(value) ? value[0] : value
	return typeof firstValue === 'string' ? firstValue : ''
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

const messages = defineMessages({
	signInTitle: {
		id: 'auth.sign-in.title',
		defaultMessage: 'Sign In',
	},
})

useHead({
	title() {
		return `${formatMessage(messages.signInTitle)} - Modrinth`
	},
})

const auth = await useAuth()
const route = useNativeRoute()
const pendingSignInOAuthProvider = useStorage<AuthProvider | null>(
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	null,
	undefined,
	{ initOnMounted: true },
)
const lastSignInOAuthProvider = useStorage<AuthProvider | null>(
	LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	null,
	undefined,
	{ initOnMounted: true },
)

if (route.query.state !== undefined) {
	await navigateTo(
		{
			path: '/auth/create/oauth',
			query: route.query,
		},
		{
			replace: true,
		},
	)
}

const redirectTarget = getQueryString(route.query.redirect)
const subtleLauncherRedirectUri = ref<string>()

if (route.query.code) {
	await finishSignIn()
}

if (auth.value.user) {
	await finishSignIn()
}

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

const email = ref('')
const password = ref('')
const token = ref('')

const flow = ref(getQueryString(route.query.flow))

async function beginPasswordSignIn() {
	pendingSignInOAuthProvider.value = null
	lastSignInOAuthProvider.value = null
	startLoading()
	try {
		const res = await client.labrinth.auth_v2.login({
			username: email.value,
			password: password.value,
			challenge: token.value,
		})

		if (res.flow) {
			flow.value = res.flow
		} else {
			await finishSignIn(res.session)
		}
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

const twoFactorCode = ref('')
async function begin2FASignIn() {
	startLoading()
	try {
		const res = await client.labrinth.auth_v2.login2FA({
			flow: flow.value,
			code: twoFactorCode.value,
		})

		await finishSignIn(res.session)
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

async function finishSignIn(sessionToken?: string | null) {
	if (route.query.launcher) {
		let token = sessionToken
		if (!token) {
			token = auth.value.token
		}

		promotePendingSignInOAuthProvider()

		const redirectUrl = `${getLauncherRedirectUrl(route)}/?code=${token}`

		if (redirectUrl.startsWith('https://launcher-files.modrinth.com/')) {
			await navigateTo(redirectUrl, {
				external: true,
			})
		} else {
			// When redirecting to localhost, the auth token is very visible in the URL to the user.
			// While we could make it harder to find with a POST request, such is security by obscurity:
			// the user and other applications would still be able to sniff the token in the request body.
			// So, to make the UX a little better by not changing the displayed URL, while keeping the
			// token hidden from very casual observation and keeping the protocol as close to OAuth's
			// standard flows as possible, let's execute the redirect within an iframe that visually
			// covers the entire page.
			subtleLauncherRedirectUri.value = redirectUrl
		}

		return
	}

	if (sessionToken) {
		await useAuth(sessionToken)
		await useUser()
		queryClient.clear()
	}

	promotePendingSignInOAuthProvider()

	if (route.query.redirect) {
		const redirect = decodeURIComponent(getQueryString(route.query.redirect))
		await navigateTo(redirect, {
			replace: true,
		})
	} else {
		await navigateTo('/dashboard')
	}
}
</script>
