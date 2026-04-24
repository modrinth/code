<template>
	<div v-if="subtleLauncherRedirectUri">
		<iframe
			:src="subtleLauncherRedirectUri"
			class="fixed left-0 top-0 z-[9999] m-0 h-full w-full border-0 p-0"
		></iframe>
	</div>
	<div
		v-else
		class="universal-card mx-auto flex w-full max-w-[28rem] flex-col gap-6 border border-solid border-surface-5"
	>
		<template v-if="flow && !subtleLauncherRedirectUri">
			<label for="two-factor-code">
				<span class="label__title">{{ formatMessage(messages.twoFactorCodeLabel) }}</span>
				<span class="label__description">
					{{ formatMessage(messages.twoFactorCodeLabelDescription) }}
				</span>
			</label>
			<StyledInput
				id="two-factor-code"
				v-model="twoFactorCodeModel"
				:maxlength="11"
				inputmode="numeric"
				:placeholder="formatMessage(messages.twoFactorCodeInputPlaceholder)"
				autocomplete="one-time-code"
				@keyup.enter="onTwoFactorSignIn()"
			/>

			<button class="btn btn-primary min-h-10 font-bold no-underline" @click="onTwoFactorSignIn()">
				{{ formatMessage(commonMessages.signInButton) }} <RightArrowIcon class="ml-2" />
			</button>
		</template>
		<template v-else>
			<div class="flex flex-col gap-6">
				<div class="text-center text-2xl font-semibold text-contrast">
					{{ formatMessage(messages.signInWithLabel) }}
				</div>

				<section class="flex flex-col gap-2.5">
					<ButtonStyled>
						<a
							class="oauth-provider-link !shadow-none"
							:class="{ 'oauth-provider-link--last': lastSignInProvider === 'google' }"
							:href="getAuthUrl('google', redirectTarget)"
							@click="onOAuthProviderClick('google')"
						>
							<GoogleColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'Google' })
							}}</span>
							<span
								v-if="lastSignInProvider === 'google'"
								class="oauth-provider-last-sign-in-badge"
							>
								{{ formatMessage(messages.lastSignInLabel) }}
							</span>
						</a>
					</ButtonStyled>
					<ButtonStyled>
						<a
							class="oauth-provider-link !shadow-none"
							:class="{ 'oauth-provider-link--last': lastSignInProvider === 'microsoft' }"
							:href="getAuthUrl('microsoft', redirectTarget)"
							@click="onOAuthProviderClick('microsoft')"
						>
							<MicrosoftColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'Microsoft' })
							}}</span>
							<span
								v-if="lastSignInProvider === 'microsoft'"
								class="oauth-provider-last-sign-in-badge"
							>
								{{ formatMessage(messages.lastSignInLabel) }}
							</span>
						</a>
					</ButtonStyled>
					<ButtonStyled>
						<a
							class="oauth-provider-link !shadow-none"
							:class="{ 'oauth-provider-link--last': lastSignInProvider === 'discord' }"
							:href="getAuthUrl('discord', redirectTarget)"
							@click="onOAuthProviderClick('discord')"
						>
							<DiscordColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'Discord' })
							}}</span>
							<span
								v-if="lastSignInProvider === 'discord'"
								class="oauth-provider-last-sign-in-badge"
							>
								{{ formatMessage(messages.lastSignInLabel) }}
							</span>
						</a>
					</ButtonStyled>
					<ButtonStyled>
						<a
							class="oauth-provider-link !shadow-none"
							:class="{ 'oauth-provider-link--last': lastSignInProvider === 'github' }"
							:href="getAuthUrl('github', redirectTarget)"
							@click="onOAuthProviderClick('github')"
						>
							<GitHubColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'GitHub' })
							}}</span>
							<span
								v-if="lastSignInProvider === 'github'"
								class="oauth-provider-last-sign-in-badge"
							>
								{{ formatMessage(messages.lastSignInLabel) }}
							</span>
						</a>
					</ButtonStyled>
					<ButtonStyled>
						<a
							class="oauth-provider-link !shadow-none"
							:class="{ 'oauth-provider-link--last': lastSignInProvider === 'gitlab' }"
							:href="getAuthUrl('gitlab', redirectTarget)"
							@click="onOAuthProviderClick('gitlab')"
						>
							<GitLabColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'GitLab' })
							}}</span>
							<span
								v-if="lastSignInProvider === 'gitlab'"
								class="oauth-provider-last-sign-in-badge"
							>
								{{ formatMessage(messages.lastSignInLabel) }}
							</span>
						</a>
					</ButtonStyled>
					<ButtonStyled>
						<a
							class="oauth-provider-link !shadow-none"
							:class="{ 'oauth-provider-link--last': lastSignInProvider === 'steam' }"
							:href="getAuthUrl('steam', redirectTarget)"
							@click="onOAuthProviderClick('steam')"
						>
							<SteamColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'Steam' })
							}}</span>
							<span v-if="lastSignInProvider === 'steam'" class="oauth-provider-last-sign-in-badge">
								{{ formatMessage(messages.lastSignInLabel) }}
							</span>
						</a>
					</ButtonStyled>
				</section>

				<div class="h-px w-full bg-surface-5"></div>

				<section class="flex flex-col gap-2.5">
					<label for="email" hidden>{{ formatMessage(commonMessages.emailUsernameLabel) }}</label>
					<StyledInput
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
					<StyledInput
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

					<ButtonStyled color="brand">
						<button
							class="!w-full"
							:disabled="globals?.captcha_enabled ? !tokenModel : false"
							@click="onPasswordSignIn()"
						>
							{{ formatMessage(messages.continueWithEmail) }} <RightArrowIcon />
						</button>
					</ButtonStyled>

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
			</div>
		</template>
	</div>
</template>

<script setup>
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
} from '@modrinth/assets'
import { ButtonStyled, commonMessages, defineMessages, StyledInput, useVIntl } from '@modrinth/ui'
import { useStorage } from '@vueuse/core'
import { computed } from 'vue'

import HCaptcha from '@/components/ui/auth/HCaptcha.vue'
import {
	getAuthUrl,
	LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
} from '@/composables/auth.ts'

const props = defineProps({
	subtleLauncherRedirectUri: {
		type: String,
		default: '',
	},
	flow: {
		default: '',
	},
	redirectTarget: {
		type: String,
		default: '',
	},
	routeQuery: {
		type: Object,
		default: () => ({}),
	},
	globals: {
		type: Object,
		default: null,
	},
	email: {
		type: String,
		default: '',
	},
	password: {
		type: String,
		default: '',
	},
	token: {
		type: String,
		default: '',
	},
	twoFactorCode: {
		default: null,
	},
	onPasswordSignIn: {
		type: Function,
		default: () => {},
	},
	onTwoFactorSignIn: {
		type: Function,
		default: () => {},
	},
	onSetCaptchaRef: {
		type: Function,
		default: undefined,
	},
})

const emit = defineEmits([
	'update:email',
	'update:password',
	'update:token',
	'update:twoFactorCode',
])

const emailModel = computed({
	get: () => props.email,
	set: (value) => emit('update:email', value),
})

const passwordModel = computed({
	get: () => props.password,
	set: (value) => emit('update:password', value),
})

const tokenModel = computed({
	get: () => props.token,
	set: (value) => emit('update:token', value),
})

const twoFactorCodeModel = computed({
	get: () => props.twoFactorCode,
	set: (value) => emit('update:twoFactorCode', value),
})

const lastSignInOAuthProvider = useStorage(LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY, null)
const pendingSignInOAuthProvider = useStorage(PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY, null)
const lastSignInProvider = computed(() => lastSignInOAuthProvider.value)
const onOAuthProviderClick = (provider) => {
	pendingSignInOAuthProvider.value = provider
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
	twoFactorCodeInputPlaceholder: {
		id: 'auth.sign-in.2fa.placeholder',
		defaultMessage: 'Enter code...',
	},
	twoFactorCodeLabel: {
		id: 'auth.sign-in.2fa.label',
		defaultMessage: 'Enter two-factor code',
	},
	twoFactorCodeLabelDescription: {
		id: 'auth.sign-in.2fa.description',
		defaultMessage: 'Please enter a two-factor code to proceed.',
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
		defaultMessage: 'Last sign in',
	},
})
</script>

<style scoped lang="scss">
.oauth-provider-link {
	position: relative;
	overflow: visible;
}

.oauth-provider-link--last {
	border-color: var(--color-green) !important;
}

.oauth-provider-last-sign-in-badge {
	position: absolute;
	top: -0.75rem;
	right: 0.5rem;
	z-index: 1;
	border-radius: 9999px;
	background-color: var(--surface-3);
	color: var(--color-green);
	border: 1px solid var(--color-green);
	padding: 0.25rem 0.5rem;
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
