<template>
	<div
		class="universal-card mx-auto flex w-full max-w-[27rem] flex-col gap-6 border border-solid border-surface-5 !p-6"
	>
		<template v-if="flow">
			<div class="flex flex-col gap-4" :aria-busy="twoFactorPending">
				<div class="flex w-full flex-col gap-1.5">
					<label for="two-factor-code">
						<span id="two-factor-label" class="label__title">
							{{ formatMessage(messages.twoFactorCodeLabel) }}
						</span>
						<span id="two-factor-description" class="label__description">
							{{ formatMessage(messages.twoFactorCodeDescription) }}
						</span>
					</label>
					<TwoFactorAuthCodeInput
						id="two-factor-code"
						ref="twoFactorInput"
						v-model="twoFactorCode"
						class="mx-auto mt-3"
						allow-backup-code
						autofocus
						:readonly="twoFactorPending"
						:error="twoFactorError"
						aria-labelledby="two-factor-label"
						:aria-describedby="
							twoFactorError ? 'two-factor-description two-factor-error' : 'two-factor-description'
						"
						@complete="begin2FASignIn"
					/>
				</div>
				<Admonition v-if="twoFactorError" id="two-factor-error" type="critical" role="alert">
					{{ formatMessage(messages.twoFactorIncorrect) }}
				</Admonition>
			</div>
		</template>
		<template v-else-if="account">
			<div class="flex flex-col items-center gap-3 text-center">
				<Avatar :src="account.avatarUrl" size="64px" circle />
				<div class="flex flex-col gap-2">
					<h1 class="m-0 text-2xl font-semibold text-contrast">
						{{ formatMessage(messages.title, { name: account.username }) }}
					</h1>
					<p v-if="account.authMethod" class="m-0 text-primary">
						{{ formatMessage(messages.lastSignedInWith, { method: account.authMethod }) }}
					</p>
				</div>
			</div>

			<template v-if="oauthProvider">
				<div class="flex flex-col gap-2.5">
					<ButtonLink
						type="colored"
						color="brand"
						class="!w-full !justify-center"
						:href="getAuthUrl(oauthProvider.id, redirectTarget)"
						:aria-label="
							formatMessage(messages.signInWithProvider, { provider: oauthProvider.name })
						"
						@click="pendingSignInOAuthProvider = oauthProvider.id"
					>
						<component :is="oauthProvider.icon" />
						{{ formatMessage(messages.signInWithProvider, { provider: oauthProvider.name }) }}
					</ButtonLink>
					<ButtonLink class="!w-full !justify-center" :to="signInAnotherWayRoute">
						{{ formatMessage(messages.signInAnotherWay) }}
					</ButtonLink>
				</div>
			</template>

			<template v-else-if="account.authMethod === 'passkey'">
				<div class="flex flex-col gap-2.5">
					<Button
						type="colored"
						color="brand"
						class="!w-full !justify-center"
						@click="beginPasskeySignIn"
					>
						<UserKeyIcon />
						{{ formatMessage(messages.continueWithPasskey) }}
					</Button>
					<ButtonLink class="!w-full !justify-center" :to="signInAnotherWayRoute">
						{{ formatMessage(messages.signInAnotherWay) }}
					</ButtonLink>
				</div>
			</template>

			<template v-else-if="account.authMethod === 'password'">
				<section class="flex w-full flex-col gap-2.5">
					<label for="reauth-username" hidden>
						{{ formatMessage(commonMessages.emailUsernameLabel) }}
					</label>
					<Input
						id="reauth-username"
						:model-value="loginIdentifier"
						:icon="MailIcon"
						type="text"
						autocomplete="username"
						disabled
						wrapper-class="w-full"
					/>

					<label for="reauth-password" hidden>
						{{ formatMessage(commonMessages.passwordLabel) }}
					</label>
					<Input
						id="reauth-password"
						v-model="password"
						:icon="KeyIcon"
						type="password"
						autocomplete="current-password"
						:placeholder="formatMessage(commonMessages.passwordLabel)"
						wrapper-class="w-full"
						@keyup.enter="beginPasswordSignIn"
					/>

					<HCaptcha
						v-if="globals?.captcha_enabled && password"
						:ref="setCaptchaRef"
						v-model="captchaToken"
					/>

					<Button
						type="colored"
						color="brand"
						class="!w-full"
						:disabled="globals?.captcha_enabled ? !captchaToken : false"
						@click="beginPasswordSignIn"
					>
						{{ formatMessage(commonMessages.signInButton) }}
						<RightArrowIcon />
					</Button>
					<ButtonLink class="!w-full !justify-center" :to="signInAnotherWayRoute">
						{{ formatMessage(messages.signInAnotherWay) }}
					</ButtonLink>
				</section>
			</template>

			<ButtonLink v-else class="!w-full !justify-center" :to="signInAnotherWayRoute">
				{{ formatMessage(messages.signInAnotherWay) }}
			</ButtonLink>
		</template>
	</div>
</template>

<script setup lang="ts">
import {
	DiscordColorIcon,
	GitHubColorIcon,
	GitLabColorIcon,
	GoogleColorIcon,
	KeyIcon,
	MailIcon,
	MicrosoftColorIcon,
	RightArrowIcon,
	SteamColorIcon,
	UserKeyIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	ButtonLink,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'
import type { LocationQueryValue } from 'vue-router'

import HCaptcha from '@/components/ui/auth/HCaptcha.vue'
import TwoFactorAuthCodeInput from '@/components/ui/auth/TwoFactorAuthCodeInput.vue'
import {
	hydrateStoredAccounts,
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	rememberStoredAccount,
	type StoredAccountAuthMethod,
	useStoredAccounts,
} from '@/composables/accounts.ts'
import {
	ADD_ACCOUNT_QUERY_PARAM,
	getAuthUrl,
	promotePendingSignInOAuthProvider,
} from '@/composables/auth.ts'
import { getPasskeyCredential } from '@/helpers/passkey.ts'

const oauthProviders = [
	{ id: 'discord', name: 'Discord', icon: DiscordColorIcon },
	{ id: 'github', name: 'GitHub', icon: GitHubColorIcon },
	{ id: 'microsoft', name: 'Microsoft', icon: MicrosoftColorIcon },
	{ id: 'google', name: 'Google', icon: GoogleColorIcon },
	{ id: 'steam', name: 'Steam', icon: SteamColorIcon },
	{ id: 'gitlab', name: 'GitLab', icon: GitLabColorIcon },
] as const

type OAuthProviderId = (typeof oauthProviders)[number]['id']

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
const storedAccounts = useStoredAccounts()

const messages = defineMessages({
	twoFactorIncorrect: {
		id: 'auth.two-factor.incorrect-code',
		defaultMessage: 'The two-factor code is incorrect. Try again or use a backup code.',
	},
	title: {
		id: 'auth.reauthenticate.title',
		defaultMessage: '{name} needs to be reauthenticated.',
	},
	lastSignedInWith: {
		id: 'auth.reauthenticate.last-signed-in-with',
		defaultMessage:
			'It was last signed into with {method, select, password {a password} passkey {a passkey} github {GitHub} discord {Discord} google {Google} microsoft {Microsoft} gitlab {GitLab} steam {Steam} paypal {PayPal} other {{method}} }.',
	},
	signInWithProvider: {
		id: 'auth.reauthenticate.sign-in-with-provider',
		defaultMessage: 'Sign in with {provider}',
	},
	signInAnotherWay: {
		id: 'auth.reauthenticate.sign-in-another-way',
		defaultMessage: 'Sign in another way',
	},
	continueWithPasskey: {
		id: 'auth.reauthenticate.continue-with-passkey',
		defaultMessage: 'Continue with passkey',
	},
	twoFactorCodeLabel: {
		id: 'auth.reauthenticate.2fa.label',
		defaultMessage: 'Two-factor authentication',
	},
	twoFactorCodeDescription: {
		id: 'auth.reauthenticate.2fa.description',
		defaultMessage:
			'Enter the 6-digit code from your authenticator app, or one of your backup codes.',
	},
	pageTitle: {
		id: 'auth.reauthenticate.page-title',
		defaultMessage: 'Reauthenticate',
	},
})

useHead({
	title() {
		return `${formatMessage(messages.pageTitle)} - Modrinth`
	},
})

const accountId = computed(() => getQueryString(route.query.account))
const account = computed(() => storedAccounts.value.find((stored) => stored.id === accountId.value))
const redirectTarget = computed(
	() =>
		getQueryString(route.query.redirect) ||
		(account.value ? `/user/${account.value.username}` : ''),
)
const loginIdentifier = computed(() => account.value?.username || '')
const oauthProvider = computed(() =>
	oauthProviders.find((provider) => provider.id === account.value?.authMethod),
)

const signInAnotherWayRoute = computed(() => ({
	path: '/auth/sign-in',
	query: {
		redirect: redirectTarget.value,
		[ADD_ACCOUNT_QUERY_PARAM]: 'true',
	},
}))

const pendingSignInOAuthProvider = useStorage<OAuthProviderId | StoredAccountAuthMethod | null>(
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	null,
	undefined,
	{ initOnMounted: true },
)

const password = ref('')
const captchaToken = ref('')
const twoFactorCode = ref('')
const twoFactorPending = ref(false)
const twoFactorError = ref(false)
const twoFactorInput = ref<InstanceType<typeof TwoFactorAuthCodeInput>>()
const flow = ref('')
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

onMounted(() => {
	hydrateStoredAccounts()
	if (!accountId.value || !account.value) {
		void navigateTo({
			path: '/auth/sign-in',
			query: { redirect: redirectTarget.value },
		})
	}
})

async function completeSignIn(sessionToken: string, authMethod: StoredAccountAuthMethod) {
	await useAuth(sessionToken)
	await useUser()
	queryClient.clear()

	const signedIn = await useAuth()
	if (signedIn.value.user && signedIn.value.token) {
		rememberStoredAccount(signedIn.value.user, signedIn.value.token, { authMethod })
	}

	promotePendingSignInOAuthProvider()
	await navigateTo(redirectTarget.value, { replace: true })
}

function notifyError(error: unknown) {
	addNotification({
		title: formatMessage(commonMessages.errorNotificationTitle),
		text: getErrorMessage(error),
		type: 'error',
	})
	captcha.value?.reset?.()
}

async function beginPasswordSignIn() {
	pendingSignInOAuthProvider.value = null
	startLoading()
	try {
		const res = await client.labrinth.auth_v2.login({
			username: loginIdentifier.value,
			password: password.value,
			challenge: captchaToken.value,
		})

		if (res.flow) {
			flow.value = res.flow
		} else if (res.session) {
			await completeSignIn(res.session, 'password')
		}
	} catch (err) {
		notifyError(err)
	}
	stopLoading()
}

async function begin2FASignIn(code: string) {
	if (twoFactorPending.value) return
	twoFactorPending.value = true
	twoFactorError.value = false
	startLoading()
	try {
		const res = await client.labrinth.auth_v2.login2FA({
			flow: flow.value,
			code,
		})
		await completeSignIn(res.session, 'password')
	} catch {
		twoFactorCode.value = ''
		twoFactorError.value = true
	} finally {
		twoFactorPending.value = false
		stopLoading()
		if (twoFactorError.value) {
			await nextTick()
			twoFactorInput.value?.focus()
		}
	}
}

async function beginPasskeySignIn() {
	startLoading()
	try {
		const start = await client.labrinth.auth_v2.authenticatePasskeyStart()
		const credential = await getPasskeyCredential(start.options.publicKey)
		const result = await client.labrinth.auth_v2.authenticatePasskeyFinish({
			flow: start.flow,
			credential,
		})
		pendingSignInOAuthProvider.value = 'passkey'
		await completeSignIn(result.session, 'passkey')
	} catch (err) {
		notifyError(err)
	}
	stopLoading()
}
</script>
