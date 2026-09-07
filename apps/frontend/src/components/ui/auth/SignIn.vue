<template>
	<div v-if="subtleLauncherRedirectUri">
		<iframe :src="subtleLauncherRedirectUri" class="hidden"></iframe>
		<div
			class="universal-card mx-auto flex w-full max-w-[27rem] flex-col gap-6 border border-solid border-surface-5 !p-6 text-center"
		>
			<div class="flex flex-col gap-2">
				<h1 class="m-0 text-2xl font-semibold text-contrast">Opening Modrinth App...</h1>
				<p class="m-0 text-left text-primary">
					If the app doesn’t open, use the button below to finish signing in.
				</p>
			</div>
			<div class="flex flex-col gap-2">
				<Button
					type="colored"
					color="brand"
					class="!w-full !justify-center"
					@click="sendLauncherCallback"
				>
					{{ formatMessage(messages.returnToLauncherButton) }}
					<RightArrowIcon />
				</Button>
				<ButtonLink to="/" class="!w-full !justify-center">
					{{ formatMessage(messages.goToWebsiteButton) }}
				</ButtonLink>
			</div>
		</div>
	</div>
	<div
		v-else
		class="universal-card mx-auto flex w-full max-w-[27rem] flex-col gap-6 border border-solid border-surface-5 !p-6"
	>
		<template v-if="flow && !subtleLauncherRedirectUri">
			<div class="flex flex-col items-end gap-4">
				<div class="flex flex-col gap-1.5">
					<label for="two-factor-code">
						<span class="label__title">{{ formatMessage(messages.twoFactorCodeLabel) }}</span>
						<span class="label__description">
							{{ formatMessage(messages.twoFactorCodeLabelDescription) }}
						</span>
					</label>
					<Input
						id="two-factor-code"
						v-model="twoFactorCodeModel"
						:maxlength="11"
						inputmode="numeric"
						:placeholder="formatMessage(messages.twoFactorCodeInputPlaceholder)"
						autocomplete="one-time-code"
						@keyup.enter="onTwoFactorSignIn()"
					/>
				</div>
				<Button type="colored" color="brand" @click="onTwoFactorSignIn()">
					{{ formatMessage(commonMessages.signInButton) }} <RightArrowIcon />
				</Button>
			</div>
		</template>
		<template v-else>
			<div class="flex flex-col gap-5">
				<template v-if="accounts.length && !addingAccount">
					<div class="flex w-full flex-col gap-4">
						<div class="text-center text-2xl font-semibold text-contrast">
							{{ formatMessage(messages.chooseAccountLabel) }}
						</div>
						<AccountChoiceList
							:accounts="accounts"
							:add-account-label="formatMessage(messages.addAccountLabel)"
							@select="emit('select', $event)"
							@add="addingAccount = true"
						/>
					</div>
				</template>
				<template v-else>
					<div class="text-center text-2xl font-semibold text-contrast">
						{{ formatMessage(messages.signInWithLabel) }}
					</div>

					<section class="grid grid-cols-1 gap-2.5 sm:grid-cols-2">
						<ButtonLink
							v-for="provider in oauthProviders"
							:key="provider.id"
							class="relative w-full !justify-center overflow-visible"
							:class="{
								'!border !border-[var(--color-green)]': lastSignInOAuthProvider === provider.id,
							}"
							:href="getAuthUrl(provider.id, redirectTarget)"
							:aria-label="
								formatMessage(messages.continueWithProvider, { provider: provider.name })
							"
							@click="onOAuthProviderClick(provider.id)"
						>
							<component :is="provider.icon" />
							<span>{{ provider.name }}</span>
							<span
								v-if="lastSignInOAuthProvider === provider.id"
								class="oauth-provider-last-sign-in-badge"
							>
								{{ formatMessage(messages.lastSignInLabel) }}
							</span>
						</ButtonLink>
						<Button
							class="relative !w-full !justify-center overflow-visible sm:col-span-2"
							:class="{
								'!border !border-[var(--color-green)]': lastSignInOAuthProvider === 'passkey',
							}"
							role="button"
							tabindex="0"
							@click="onPasskeySignIn"
							@keydown.enter="onPasskeySignIn"
						>
							<UserKeyIcon />
							<span class="ml-1">{{ formatMessage(messages.continueWithPasskey) }}</span>
							<span
								v-if="lastSignInOAuthProvider === 'passkey'"
								class="oauth-provider-last-sign-in-badge"
							>
								{{ formatMessage(messages.lastSignInLabel) }}
							</span>
						</Button>
					</section>

					<div class="h-px w-full bg-surface-5"></div>

					<section class="mx-auto flex w-full flex-col gap-2.5">
						<label for="email" hidden>{{ formatMessage(commonMessages.emailUsernameLabel) }}</label>
						<Input
							id="email"
							v-model="emailModel"
							:icon="MailIcon"
							type="text"
							inputmode="email"
							autocomplete="username"
							:placeholder="formatMessage(commonMessages.emailUsernameLabel)"
							wrapper-class="w-full"
						/>

						<label for="password" hidden>{{ formatMessage(commonMessages.passwordLabel) }}</label>
						<Input
							id="password"
							v-model="passwordModel"
							:icon="KeyIcon"
							type="password"
							autocomplete="current-password"
							:placeholder="formatMessage(commonMessages.passwordLabel)"
							wrapper-class="w-full"
						/>

						<HCaptcha
							v-if="globals?.captcha_enabled && emailModel && passwordModel"
							:ref="onSetCaptchaRef"
							v-model="tokenModel"
						/>

						<Button
							type="colored"
							color="brand"
							class="!w-full"
							:disabled="globals?.captcha_enabled ? !tokenModel : false"
							@click="onPasswordSignIn()"
						>
							{{ formatMessage(messages.continueWithEmail) }} <RightArrowIcon />
						</Button>

						<div class="flex flex-wrap items-center justify-center gap-2.5 !text-base">
							<NuxtLink
								class="text-link"
								:to="{
									path: '/auth/reset-password',
									query: routeQuery,
								}"
							>
								{{ formatMessage(messages.forgotPasswordLabel) }}
							</NuxtLink>
							<div class="h-1.5 w-1.5 rounded-full bg-surface-5" />
							<NuxtLink
								class="inline text-link"
								:to="{
									path: '/auth/sign-up',
									query: routeQuery,
								}"
							>
								{{ formatMessage(messages.createAccountLabel) }}
							</NuxtLink>
						</div>
					</section>
				</template>
			</div>
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
	type AccountChoice,
	AccountChoiceList,
	Button,
	ButtonLink,
	commonMessages,
	defineMessages,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { useStorage } from '@vueuse/core'
import { ref } from 'vue'
import type { LocationQuery } from 'vue-router'

import HCaptcha from '@/components/ui/auth/HCaptcha.vue'
import {
	LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
} from '@/composables/accounts.ts'
import { getAuthUrl } from '@/composables/auth.ts'

const oauthProviders = [
	{ id: 'discord', name: 'Discord', icon: DiscordColorIcon },
	{ id: 'github', name: 'GitHub', icon: GitHubColorIcon },
	{ id: 'microsoft', name: 'Microsoft', icon: MicrosoftColorIcon },
	{ id: 'google', name: 'Google', icon: GoogleColorIcon },
	{ id: 'steam', name: 'Steam', icon: SteamColorIcon },
	{ id: 'gitlab', name: 'GitLab', icon: GitLabColorIcon },
] as const

type AuthProvider = (typeof oauthProviders)[number]['id'] | 'passkey'

interface AuthGlobals {
	captcha_enabled?: boolean
	[key: string]: unknown
}

interface Props {
	subtleLauncherRedirectUri?: string
	flow?: string
	redirectTarget?: string
	routeQuery?: LocationQuery
	globals?: AuthGlobals | null
	onPasswordSignIn?: () => void
	onTwoFactorSignIn?: () => void
	onPasskeySignIn?: () => void
	onSetCaptchaRef?: ((captchaRef: unknown) => void) | undefined
	accounts?: AccountChoice[]
}

const {
	subtleLauncherRedirectUri = '',
	flow = '',
	redirectTarget = '',
	routeQuery = {},
	globals = null,
	onPasswordSignIn = () => {},
	onTwoFactorSignIn = () => {},
	onPasskeySignIn = () => {},
	onSetCaptchaRef = undefined,
	accounts = [],
} = defineProps<Props>()

const addingAccount = ref(false)

const emit = defineEmits<{
	select: [account: AccountChoice]
}>()

const emailModel = defineModel<string>('email', { default: '' })
const passwordModel = defineModel<string>('password', { default: '' })
const tokenModel = defineModel<string>('token', { default: '' })
const twoFactorCodeModel = defineModel<string>('twoFactorCode', { default: '' })

const lastSignInOAuthProvider = useStorage<AuthProvider | null>(
	LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	null,
	undefined,
	{ initOnMounted: true },
)
const pendingSignInOAuthProvider = useStorage<AuthProvider | null>(
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	null,
	undefined,
	{ initOnMounted: true },
)
const onOAuthProviderClick = (provider: AuthProvider) => {
	pendingSignInOAuthProvider.value = provider
}

async function sendLauncherCallback() {
	await fetch(subtleLauncherRedirectUri, { mode: 'no-cors' }).catch(() => undefined)
}

const { formatMessage } = useVIntl()

const messages = defineMessages({
	forgotPasswordLabel: {
		id: 'auth.sign-in.forgot-password',
		defaultMessage: 'Forgot password',
	},
	noAccountLabel: {
		id: 'auth.sign-in.no-account',
		defaultMessage: "Don't have an account?",
	},
	createAccountLabel: {
		id: 'auth.sign-in.create-account',
		defaultMessage: 'Sign up',
	},
	signInWithLabel: {
		id: 'auth.sign-in.sign-in-with',
		defaultMessage: 'Sign into Modrinth',
	},
	chooseAccountLabel: {
		id: 'auth.sign-in.choose-account',
		defaultMessage: 'Choose an account to use in Modrinth App',
	},
	addAccountLabel: {
		id: 'auth.sign-in.add-account',
		defaultMessage: 'Add account',
	},
	twoFactorCodeInputPlaceholder: {
		id: 'auth.sign-in.2fa.placeholder',
		defaultMessage: 'Enter code...',
	},
	twoFactorCodeLabel: {
		id: 'auth.sign-in.2fa.label',
		defaultMessage: 'Two-factor authentication',
	},
	twoFactorCodeLabelDescription: {
		id: 'auth.sign-in.2fa.description',
		defaultMessage:
			'Enter the 6-digit code from your authenticator app, or one of your backup codes.',
	},
	continueWithProvider: {
		id: 'auth.continue-with-provider',
		defaultMessage: 'Continue with {provider}',
	},
	continueWithEmail: {
		id: 'auth.sign-in.continue-with-email',
		defaultMessage: 'Continue with Email',
	},
	lastSignInLabel: {
		id: 'auth.sign-in.last-sign-in',
		defaultMessage: 'Last used',
	},
	continueWithPasskey: {
		id: 'auth.sign-in.continue-with-passkey',
		defaultMessage: 'Continue with passkey',
	},
	launcherSignInCompleteTitle: {
		id: 'auth.sign-in.launcher.complete.title',
		defaultMessage: 'You’re signed in',
	},
	launcherSignInCompleteDescription: {
		id: 'auth.sign-in.launcher.complete.description',
		defaultMessage:
			'We’re returning you to the Modrinth App. If nothing happens, use the button below.',
	},
	returnToLauncherButton: {
		id: 'auth.sign-in.launcher.complete.return-button',
		defaultMessage: 'Open Modrinth App',
	},
	goToWebsiteButton: {
		id: 'auth.sign-in.launcher.complete.go-to-website',
		defaultMessage: 'Go to Modrinth.com',
	},
})
</script>

<style scoped lang="scss">
.oauth-provider-last-sign-in-badge {
	position: absolute;
	top: -0.75rem;
	right: 0.25rem;
	z-index: 1;
	border-radius: 9999px;
	background-color: var(--surface-3);
	color: var(--color-green);
	border: 1px solid var(--color-green);
	padding: 0.125rem 0.375rem;
	font-size: 0.75rem;
	font-weight: 600;
	line-height: 1;
	pointer-events: none;
}
.oauth-provider-last-sign-in-badge::before {
	content: '';
	inset: 0;
	border-radius: inherit;
	background-color: var(--color-green-highlight);
	position: absolute;
	z-index: 0;
}
</style>
