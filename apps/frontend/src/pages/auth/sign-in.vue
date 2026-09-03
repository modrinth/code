<template>
	<SignInView
		v-if="signInReady || subtleLauncherRedirectUri"
		v-model:email="email"
		v-model:password="password"
		v-model:token="token"
		v-model:two-factor-code="twoFactorCode"
		:subtle-launcher-redirect-uri="subtleLauncherRedirectUri"
		:flow="flow"
		:redirect-target="redirectTarget"
		:route-query="route.query"
		:globals="globals"
		:accounts="launcherAccountChoices"
		:on-password-sign-in="beginPasswordSignIn"
		:on-two-factor-sign-in="begin2FASignIn"
		:on-passkey-sign-in="beginPasskeySignin"
		:on-set-captcha-ref="setCaptchaRef"
		@select="onSelectLauncherAccount"
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
	hydrateStoredAccounts,
	isStoredAccountAuthMethod,
	LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	rememberStoredAccount,
	type StoredAccount,
	type StoredAccountAuthMethod,
	useStoredAccounts,
} from '@/composables/accounts.ts'
import {
	ADD_ACCOUNT_QUERY_PARAM,
	getLauncherRedirectUrl,
	promotePendingSignInOAuthProvider,
} from '@/composables/auth.ts'
import { getPasskeyCredential } from '@/helpers/passkey.ts'

type AuthProvider = 'discord' | 'google' | 'github' | 'gitlab' | 'steam' | 'microsoft' | 'passkey'

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

const isAddingAccount = route.query[ADD_ACCOUNT_QUERY_PARAM] !== undefined
const isLauncherSignIn = route.query.launcher !== undefined
const storedAccounts = useStoredAccounts()
const signInReady = ref(!isLauncherSignIn)

const choosableAccounts = computed((): StoredAccount[] => {
	const user = auth.value.user
	const token = auth.value.token
	const accounts = storedAccounts.value.map((stored) => {
		if (user && token && stored.id === user.id) {
			return {
				...stored,
				username: user.username,
				avatarUrl: user.avatar_url ?? stored.avatarUrl,
				token,
				role: user.role,
			}
		}

		return stored
	})

	if (user && token && !accounts.some((account) => account.id === user.id)) {
		accounts.push({
			id: user.id,
			username: user.username,
			avatarUrl: user.avatar_url ?? null,
			token,
			role: user.role,
		})
	}

	return accounts
})

const launcherAccountChoices = computed(() => {
	if (!isLauncherSignIn) return []

	const minimumAccounts = isAddingAccount ? 1 : 2
	return choosableAccounts.value.length >= minimumAccounts ? choosableAccounts.value : []
})

if (auth.value.user && !isAddingAccount && !isLauncherSignIn) {
	await finishSignIn()
}

onMounted(async () => {
	if (!isLauncherSignIn) {
		return
	}

	hydrateStoredAccounts()

	if (subtleLauncherRedirectUri.value) {
		signInReady.value = true
		return
	}

	if (
		auth.value.user &&
		!isAddingAccount &&
		choosableAccounts.value.length === 1 &&
		route.query.code === undefined
	) {
		await showLauncherOpeningPage(auth.value.token)
		if (subtleLauncherRedirectUri.value) {
			signInReady.value = true
		}
		return
	}

	signInReady.value = true
})

function getLauncherCallbackUrl(sessionToken: string) {
	return `${getLauncherRedirectUrl(route)}/?code=${sessionToken}`
}

async function showLauncherOpeningPage(sessionToken: string) {
	promotePendingSignInOAuthProvider()

	const redirectUrl = getLauncherCallbackUrl(sessionToken)

	if (redirectUrl.startsWith('https://launcher-files.modrinth.com/')) {
		await navigateTo(redirectUrl, {
			external: true,
		})
		return
	}

	subtleLauncherRedirectUri.value = redirectUrl
}

function onSelectLauncherAccount(account: { id: string }) {
	const stored = choosableAccounts.value.find((choice) => choice.id === account.id)
	if (stored) {
		void showLauncherOpeningPage(stored.token)
	}
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
			await finishSignIn(res.session, 'password')
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

		await finishSignIn(res.session, 'password')
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

async function beginPasskeySignin() {
	startLoading()
	try {
		const start = await client.labrinth.auth_v2.authenticatePasskeyStart()

		const credential = await getPasskeyCredential(start.options.publicKey)

		const result = await client.labrinth.auth_v2.authenticatePasskeyFinish({
			flow: start.flow,
			credential,
		})

		pendingSignInOAuthProvider.value = 'passkey'
		await finishSignIn(result.session, 'passkey')
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: getErrorMessage(err),
			type: 'error',
		})
	}
	stopLoading()
}

async function finishSignIn(sessionToken?: string | null, authMethod?: StoredAccountAuthMethod) {
	if (route.query.launcher) {
		const token = sessionToken ?? auth.value.token
		if (token) {
			await showLauncherOpeningPage(token)
		}

		return
	}

	if (sessionToken) {
		await useAuth(sessionToken)
		await useUser()
		queryClient.clear()
	}

	const signedIn = await useAuth()
	if (signedIn.value.user && signedIn.value.token) {
		const nextAuthMethod =
			authMethod ??
			(isStoredAccountAuthMethod(pendingSignInOAuthProvider.value)
				? pendingSignInOAuthProvider.value
				: undefined)
		rememberStoredAccount(
			signedIn.value.user,
			signedIn.value.token,
			nextAuthMethod ? { authMethod: nextAuthMethod } : undefined,
		)
	}

	promotePendingSignInOAuthProvider()

	if (route.query.redirect) {
		const redirect = decodeURIComponent(getQueryString(route.query.redirect))
		await navigateTo(redirect, {
			replace: true,
		})
	} else if (signedIn.value.user) {
		await navigateTo(`/user/${signedIn.value.user.username}`)
	}
}
</script>
