<template>
	<div class="flex flex-col gap-6">
		<div class="text-center text-2xl font-semibold text-contrast">{{ formatMessage(messages.signUpWithTitle) }}</div>
		<section class="flex flex-col gap-2.5">
			<ButtonStyled>
				<a class="!shadow-none" :href="getAuthUrl('google', redirectTarget)">
					<GoogleColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Google' }) }}</span>
				</a>
			</ButtonStyled>
			<ButtonStyled>
				<a class="!shadow-none" :href="getAuthUrl('microsoft', redirectTarget)">
					<MicrosoftColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Microsoft' }) }}</span>
				</a>
			</ButtonStyled>
			<ButtonStyled>
				<a class="!shadow-none" :href="getAuthUrl('discord', redirectTarget)">
					<DiscordColorIcon />
					<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Discord' }) }}</span>
				</a>
			</ButtonStyled>
			<template v-if="showOtherOptions">
				<ButtonStyled>
					<a class="!shadow-none" :href="getAuthUrl('github', redirectTarget)">
						<GitHubColorIcon />
						<span>{{ formatMessage(messages.continueWithProvider, { provider: 'GitHub' }) }}</span>
					</a>
				</ButtonStyled>
				<ButtonStyled>
					<a class="!shadow-none" :href="getAuthUrl('gitlab', redirectTarget)">
						<GitLabColorIcon />
						<span>{{ formatMessage(messages.continueWithProvider, { provider: 'GitLab' }) }}</span>
					</a>
				</ButtonStyled>
				<ButtonStyled>
					<a class="!shadow-none" :href="getAuthUrl('steam', redirectTarget)">
						<SteamColorIcon />
						<span>{{ formatMessage(messages.continueWithProvider, { provider: 'Steam' }) }}</span>
					</a>
				</ButtonStyled>
			</template>
			<button
				class="mx-auto -mb-3 bg-transparent pt-1 text-center text-base font-semibold text-secondary transition-all hover:text-primary"
				@click="showOtherOptions = !showOtherOptions"
			>
				{{ showOtherOptions ? formatMessage(messages.showFewerOptions) : formatMessage(messages.showOtherOptions) }}
			</button>
		</section>

		<div class="h-px w-full bg-surface-5"></div>

		<section class="flex flex-col gap-2.5">
			<label for="email" hidden>{{ formatMessage(commonMessages.emailLabel) }}</label>
			<StyledInput
				id="email"
				v-model="email"
				:icon="MailIcon"
				type="email"
				autocomplete="username"
				:placeholder="formatMessage(commonMessages.emailLabel)"
				wrapper-class="w-full"
			/>

			<label for="username" hidden>{{ formatMessage(commonMessages.usernameLabel) }}</label>
			<StyledInput
				id="username"
				v-model="username"
				:icon="UserIcon"
				type="text"
				autocomplete="username"
				:placeholder="formatMessage(commonMessages.usernameLabel)"
				wrapper-class="w-full"
			/>

			<label for="password" hidden>{{ formatMessage(commonMessages.passwordLabel) }}</label>
			<StyledInput
				id="password"
				v-model="password"
				:icon="KeyIcon"
				type="password"
				autocomplete="new-password"
				:placeholder="formatMessage(commonMessages.passwordLabel)"
				wrapper-class="w-full"
			/>

			<label for="confirm-password" hidden>{{ formatMessage(commonMessages.passwordLabel) }}</label>
			<StyledInput
				id="confirm-password"
				v-model="confirmPassword"
				:icon="KeyIcon"
				type="password"
				autocomplete="new-password"
				:placeholder="formatMessage(commonMessages.confirmPasswordLabel)"
				wrapper-class="w-full"
			/>

			<Checkbox
				v-model="subscribe"
				class="subscribe-btn"
				:label="formatMessage(messages.subscribeLabel)"
				:description="formatMessage(messages.subscribeLabel)"
			/>

			<p v-if="!route.query.launcher">
				<IntlFormatted :message-id="messages.legalDisclaimer">
					<template #terms-link="{ children }">
						<NuxtLink to="/legal/terms" class="text-link">
							<component :is="() => children" />
						</NuxtLink>
					</template>
					<template #privacy-policy-link="{ children }">
						<NuxtLink to="/legal/privacy" class="text-link">
							<component :is="() => children" />
						</NuxtLink>
					</template>
				</IntlFormatted>
			</p>

			<HCaptcha
				v-if="globals?.captcha_enabled && email && password && confirmPassword && username"
				ref="captcha"
				v-model="token"
			/>

			<ButtonStyled color="brand">
				<button
					class="!w-full"
					:disabled="globals?.captcha_enabled ? !token : false"
					@click="createAccount"
				>
					{{ formatMessage(messages.continueWithEmail) }} <RightArrowIcon />
				</button>
			</ButtonStyled>

			<div class="auth-form__additional-options">
				{{ formatMessage(messages.alreadyHaveAccountLabel) }}
				<NuxtLink
					class="text-link"
					:to="{
						path: '/auth/sign-in',
						query: route.query,
					}"
				>
					{{ formatMessage(commonMessages.signInButton) }}
				</NuxtLink>
			</div>
		</section>
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
	UserIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Checkbox,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	IntlFormatted,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import HCaptcha from '@/components/ui/HCaptcha.vue'
import { getAuthUrl } from '@/composables/auth.js'

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'auth.sign-up.title',
		defaultMessage: 'Sign Up',
	},
	signUpWithTitle: {
		id: 'auth.sign-up.title.sign-up-with',
		defaultMessage: 'Create an Account',
	},
	subscribeLabel: {
		id: 'auth.sign-up.subscribe.label',
		defaultMessage: 'Subscribe to updates about Modrinth',
	},
	legalDisclaimer: {
		id: 'auth.sign-up.legal-dislaimer',
		defaultMessage:
			"By creating an account, you agree to Modrinth's <terms-link>Terms</terms-link> and <privacy-policy-link>Privacy Policy</privacy-policy-link>.",
	},
	createAccountButton: {
		id: 'auth.sign-up.action.create-account',
		defaultMessage: 'Create account',
	},
	alreadyHaveAccountLabel: {
		id: 'auth.sign-up.sign-in-option.title',
		defaultMessage: 'Already have an account?',
	},
	continueWithProvider: {
		id: 'auth.continue-with-provider',
		defaultMessage: 'Continue with {provider}',
	},
	continueWithEmail: {
		id: 'auth.sign-up.continue-with-email',
		defaultMessage: 'Continue with Email',
	},
	showFewerOptions: {
		id: 'auth.sign-up.show-fewer-options',
		defaultMessage: 'Show fewer options',
	},
	showOtherOptions: {
		id: 'auth.sign-up.show-other-options',
		defaultMessage: 'Show other options',
	},
})

useHead({
	title: () => `${formatMessage(messages.title)} - Modrinth`,
})

const auth = await useAuth()
const route = useNativeRoute()

const redirectTarget = route.query.redirect
const showOtherOptions = ref(false)

if (auth.value.user) {
	await navigateTo('/dashboard')
}

const captcha = ref()

const { data: globals } = useQuery({
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
const username = ref('')
const password = ref('')
const confirmPassword = ref('')
const token = ref('')
const subscribe = ref(false)

async function createAccount() {
	startLoading()
	try {
		if (confirmPassword.value !== password.value) {
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: formatMessage({
					id: 'auth.sign-up.notification.password-mismatch.text',
					defaultMessage: 'Passwords do not match!',
				}),
				type: 'error',
			})
			captcha.value?.reset()
		}

		const res = await client.labrinth.auth_v2.createAccount({
			username: username.value,
			password: password.value,
			email: email.value,
			challenge: token.value,
			sign_up_newsletter: subscribe.value,
		})

		await useAuth(res.session)
		await useUser()

		if (route.query.launcher) {
			await navigateTo({ path: '/auth/sign-in', query: route.query })
			return
		}

		if (route.query.redirect) {
			await navigateTo(route.query.redirect)
		} else {
			await navigateTo('/dashboard')
		}
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
		captcha.value?.reset()
	}
	stopLoading()
}
</script>
