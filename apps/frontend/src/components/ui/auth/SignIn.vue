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
						<a class="!shadow-none" :href="getAuthUrl('google', redirectTarget)">
							<GoogleColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'Google' })
							}}</span>
						</a>
					</ButtonStyled>
					<ButtonStyled>
						<a class="!shadow-none" :href="getAuthUrl('microsoft', redirectTarget)">
							<MicrosoftColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'Microsoft' })
							}}</span>
						</a>
					</ButtonStyled>
					<ButtonStyled>
						<a class="!shadow-none" :href="getAuthUrl('discord', redirectTarget)">
							<DiscordColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'Discord' })
							}}</span>
						</a>
					</ButtonStyled>
					<ButtonStyled>
						<a class="!shadow-none" :href="getAuthUrl('github', redirectTarget)">
							<GitHubColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'GitHub' })
							}}</span>
						</a>
					</ButtonStyled>
					<ButtonStyled>
						<a class="!shadow-none" :href="getAuthUrl('gitlab', redirectTarget)">
							<GitLabColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'GitLab' })
							}}</span>
						</a>
					</ButtonStyled>
					<ButtonStyled>
						<a class="!shadow-none" :href="getAuthUrl('steam', redirectTarget)">
							<SteamColorIcon />
							<span class="ml-1">{{
								formatMessage(messages.continueWithProvider, { provider: 'Steam' })
							}}</span>
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
						<IntlFormatted :message-id="messages.additionalOptionsLabel">
							<template #forgot-password-link="{ children }">
								<NuxtLink
									class="text-link"
									:to="{
										path: '/auth/reset-password',
										query: routeQuery,
									}"
								>
									<component :is="() => children" />
								</NuxtLink>
							</template>
							<template #create-account-link="{ children }">
								<NuxtLink
									class="inline text-link"
									:to="{
										path: '/auth/sign-up',
										query: routeQuery,
									}"
								>
									<component :is="() => children" />
								</NuxtLink>
							</template>
						</IntlFormatted>
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
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	IntlFormatted,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import HCaptcha from '@/components/ui/auth/HCaptcha.vue'
import { getAuthUrl } from '@/composables/auth.ts'

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

const { formatMessage } = useVIntl()

const messages = defineMessages({
	additionalOptionsLabel: {
		id: 'auth.sign-in.additional-options',
		defaultMessage:
			"<forgot-password-link>Forgot password</forgot-password-link> • Don't have an account? <create-account-link>Sign up</create-account-link>",
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
})
</script>
