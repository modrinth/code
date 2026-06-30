<template>
	<div v-if="subtleLauncherRedirectUri">
		<iframe
			:src="subtleLauncherRedirectUri"
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
import type { LocationQueryValue } from 'vue-router'

import CreateAccountView from '@/components/ui/auth/CreateAccount.vue'
import { getLauncherRedirectUrl, promotePendingSignInOAuthProvider } from '@/composables/auth.ts'

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
const subtleLauncherRedirectUri = ref<string>()

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

async function completeOAuthSignUp() {
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
			subtleLauncherRedirectUri.value = redirectUrl
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
		const redirect = decodeURIComponent(getQueryString(route.query.redirect))
		await navigateTo(redirect, {
			replace: true,
		})
	} else {
		await navigateTo('/dashboard')
	}
}
</script>
