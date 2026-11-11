<template>
	<LauncherOpening
		v-if="launcherHandoff"
		:localhost-url="launcherHandoff.localhostUrl ?? undefined"
		:deeplink-url="launcherHandoff.deeplinkUrl ?? undefined"
	/>
	<AuthCard v-else>
		<OpeningAuthProvider
			v-if="openingProviderName"
			:provider="openingProviderName"
			@cancel="cancelProviderOpening"
		/>
		<template v-else-if="flow">
			<div class="flex flex-col gap-4" :aria-busy="twoFactorPending">
				<div class="flex w-full flex-col gap-1.5">
					<label for="two-factor-code">
						<span id="two-factor-label" class="label__title">
							{{ formatMessage(messages.twoFactorCodeLabel) }}
						</span>
						<span id="two-factor-description" class="label__description">
							{{ formatMessage(messages.twoFactorCodeLabelDescription) }}
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
							@select="onSelectLauncherAccount"
							@add="addingAccount = true"
						/>
					</div>
				</template>
				<template v-else-if="focusedReauthAccount">
					<div class="text-center text-2xl font-semibold text-contrast">
						{{ formatMessage(messages.launcherReauthTitle) }}
					</div>
					<div class="flex items-center justify-center gap-2 text-center">
						<Avatar :src="focusedReauthAccount.avatarUrl" size="36px" circle alt="" />
						<div class="text-lg font-medium text-contrast">
							{{ focusedReauthAccount.username }}
						</div>
					</div>

					<div class="flex flex-col gap-3">
						<ButtonLink
							v-if="focusedOauthProvider"
							type="colored"
							color="brand"
							class="!w-full !justify-center"
							:href="getAuthUrl(focusedOauthProvider.id, redirectTarget)"
							:aria-label="
								formatMessage(messages.continueWithProvider, {
									provider: focusedOauthProvider.name,
								})
							"
							@click="onOAuthProviderClick(focusedOauthProvider.id, $event)"
						>
							<component :is="focusedOauthProvider.icon" />
							{{
								formatMessage(messages.continueWithProvider, {
									provider: focusedOauthProvider.name,
								})
							}}
						</ButtonLink>
						<Button
							v-else-if="focusedReauthAccount.authMethod === 'passkey'"
							type="colored"
							color="brand"
							class="!w-full !justify-center"
							@click="beginPasskeySignin"
						>
							<UserKeyIcon />
							{{ formatMessage(messages.continueWithPasskey) }}
						</Button>
						<section v-else class="mx-auto flex w-full flex-col gap-2.5">
							<label for="launcher-reauth-username" class="sr-only">
								{{ formatMessage(commonMessages.emailUsernameLabel) }}
							</label>
							<input
								id="launcher-reauth-username"
								class="sr-only"
								type="text"
								autocomplete="username"
								:value="email"
								tabindex="-1"
							/>

							<PasswordSignInForm
								v-model:password="password"
								v-model:token="token"
								input-id="launcher-reauth-password"
								:submit-label="formatMessage(commonMessages.continueButton)"
								:captcha-enabled="globals?.captcha_enabled"
								:show-captcha="Boolean(password)"
								:on-submit="beginPasswordSignIn"
								:on-set-captcha-ref="setCaptchaRef"
							/>
						</section>

						<Button class="!w-full !justify-center" @click="cancelReauth">
							{{ formatMessage(messages.useDifferentAccount) }}
						</Button>
					</div>
				</template>
				<template v-else>
					<div class="text-center text-2xl font-semibold text-contrast">
						{{
							formatMessage(addingAccount ? messages.launcherReauthTitle : messages.signInWithLabel)
						}}
					</div>

					<section class="grid grid-cols-1 gap-2.5 sm:grid-cols-2">
						<ButtonLink
							v-for="provider in oauthProviders"
							:key="provider.id"
							class="relative w-full !justify-center overflow-visible"
							:class="{
								'!border !border-[var(--color-green)]': lastSignInOAuthProvider === provider.id,
							}"
							:href="getAuthUrl(provider.id, redirectTarget, requestsAppSession)"
							:aria-label="
								formatMessage(messages.continueWithProvider, { provider: provider.name })
							"
							@click="onOAuthProviderClick(provider.id, $event)"
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
							@click="beginPasskeySignin"
							@keydown.enter="beginPasskeySignin"
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
							v-model="email"
							:icon="MailIcon"
							type="text"
							inputmode="email"
							autocomplete="username"
							:placeholder="formatMessage(commonMessages.emailUsernameLabel)"
							wrapper-class="w-full"
						/>

						<PasswordSignInForm
							v-model:password="password"
							v-model:token="token"
							input-id="password"
							:submit-label="formatMessage(messages.continueWithEmail)"
							:captcha-enabled="globals?.captcha_enabled"
							:show-captcha="Boolean(email && password)"
							:on-submit="beginPasswordSignIn"
							:on-set-captcha-ref="setCaptchaRef"
						/>

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
							<button v-if="embedded" type="button" class="inline text-link" @click="openSignUp">
								{{ formatMessage(messages.createAccountLabel) }}
							</button>
							<NuxtLink
								v-else
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

					<Button
						v-if="addingAccount && accounts.length"
						class="!w-full !justify-center"
						@click="cancelAddAccount"
					>
						{{ formatMessage(messages.useExistingAccount) }}
					</Button>
				</template>
			</div>
		</template>
	</AuthCard>
</template>

<script setup lang="ts">
import { MailIcon, UserKeyIcon } from '@modrinth/assets'
import {
	AccountChoiceList,
	Admonition,
	Avatar,
	Button,
	ButtonLink,
	commonMessages,
	defineMessages,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { computed, nextTick, ref, watch } from 'vue'

import AuthCard from '@/components/ui/auth/AuthCard.vue'
import LauncherOpening from '@/components/ui/auth/LauncherOpening.vue'
import OpeningAuthProvider from '@/components/ui/auth/OpeningAuthProvider.vue'
import PasswordSignInForm from '@/components/ui/auth/PasswordSignInForm.vue'
import TwoFactorAuthCodeInput from '@/components/ui/auth/TwoFactorAuthCodeInput.vue'
import { useLastSignInAuthMethod } from '@/composables/accounts.ts'
import { getAuthUrl } from '@/composables/auth.ts'
import { isAuthEmbedded, useAuthNavigation } from '@/composables/auth-modal.ts'
import { useProviderOpening } from '@/composables/external-auth.ts'
import {
	isLauncherProtocolV2,
	LAUNCHER_REAUTH_ACCOUNT_STORAGE_KEY,
} from '@/composables/launcher-auth.ts'
import { type OAuthProviderId, oauthProviders } from '@/composables/oauth-providers.ts'
import { useSignIn } from '@/composables/sign-in.ts'

const {
	launcherHandoff,
	flow,
	twoFactorCode,
	twoFactorPending,
	twoFactorError,
	accounts,
	reauthAccount,
	redirectTarget,
	routeQuery,
	globals,
	email,
	password,
	token,
	beginPasswordSignIn,
	begin2FASignIn,
	beginPasskeySignin,
	setCaptchaRef,
	cancelReauth,
	onSelectLauncherAccount,
} = useSignIn()

const addingAccount = ref(false)

function cancelAddAccount() {
	addingAccount.value = false
	email.value = ''
	password.value = ''
	token.value = ''
}

const twoFactorInput = ref<InstanceType<typeof TwoFactorAuthCodeInput>>()

watch(twoFactorPending, async (pending) => {
	if (!pending && twoFactorError.value) {
		await nextTick()
		twoFactorInput.value?.focus()
	}
})

const lastSignInOAuthProvider = useLastSignInAuthMethod()
const focusedOauthProvider = computed(() =>
	oauthProviders.find((provider) => provider.id === reauthAccount.value?.authMethod),
)
const focusedReauthAccount = computed(() => {
	const account = reauthAccount.value
	if (!account?.authMethod) {
		return null
	}
	if (
		account.authMethod === 'password' ||
		account.authMethod === 'passkey' ||
		focusedOauthProvider.value
	) {
		return account
	}
	return null
})
const requestsAppSession = computed(
	() => isLauncherProtocolV2({ query: routeQuery.value }) && !reauthAccount.value,
)
const { openingProviderName, beginProviderHandoff, cancelProviderOpening } = useProviderOpening()
const { go } = useAuthNavigation()
const embedded = isAuthEmbedded()

function openSignUp() {
	void go({
		path: '/auth/sign-up',
		query: routeQuery.value,
	})
}

const onOAuthProviderClick = (provider: OAuthProviderId, event: MouseEvent) => {
	beginProviderHandoff(
		provider,
		event,
		getAuthUrl(provider, redirectTarget, requestsAppSession.value),
		{
			skip: Boolean(routeQuery.value.launcher || requestsAppSession.value),
			prepare() {
				const account = reauthAccount.value
				if (!account || !import.meta.client) {
					return
				}
				window.sessionStorage.setItem(LAUNCHER_REAUTH_ACCOUNT_STORAGE_KEY, account.id)
			},
		},
	)
}

const { formatMessage } = useVIntl()

const messages = defineMessages({
	twoFactorIncorrect: {
		id: 'auth.two-factor.incorrect-code',
		defaultMessage: 'The two-factor code is incorrect. Try again or use a backup code.',
	},
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
	launcherReauthTitle: {
		id: 'auth.sign-in.launcher.reauthenticate.title',
		defaultMessage: 'Sign into Modrinth App',
	},
	useDifferentAccount: {
		id: 'auth.sign-in.reauthenticate.use-different-account',
		defaultMessage: 'Use a different account',
	},
	chooseAccountLabel: {
		id: 'auth.sign-in.choose-account',
		defaultMessage: 'Choose an account to use in Modrinth App',
	},
	addAccountLabel: {
		id: 'auth.sign-in.add-account',
		defaultMessage: 'Add account',
	},
	useExistingAccount: {
		id: 'auth.sign-in.choose-existing-account',
		defaultMessage: 'Use an existing account',
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
