<template>
	<div>
		<h1>{{ formatMessage(messages.signUpWithTitle) }}</h1>

		<section class="third-party">
			<a class="btn discord-btn" :href="getAuthUrl('discord', redirectTarget)">
				<DiscordColorIcon />
				<span>Discord</span>
			</a>
			<a class="btn" :href="getAuthUrl('github', redirectTarget)">
				<GitHubColorIcon />
				<span>GitHub</span>
			</a>
			<a class="btn" :href="getAuthUrl('microsoft', redirectTarget)">
				<MicrosoftColorIcon />
				<span>Microsoft</span>
			</a>
			<a class="btn" :href="getAuthUrl('google', redirectTarget)">
				<GoogleColorIcon />
				<span>Google</span>
			</a>
			<a class="btn" :href="getAuthUrl('steam', redirectTarget)">
				<SteamColorIcon />
				<span>Steam</span>
			</a>
			<a class="btn" :href="getAuthUrl('gitlab', redirectTarget)">
				<GitLabColorIcon />
				<span>GitLab</span>
			</a>
		</section>

		<h1>{{ formatMessage(messages.createAccountTitle) }}</h1>

		<section class="auth-form">
			<div class="iconified-input">
				<label for="email" hidden>{{ formatMessage(commonMessages.emailLabel) }}</label>
				<MailIcon />
				<input
					id="email"
					v-model="email"
					type="email"
					autocomplete="username"
					class="auth-form__input"
					:placeholder="formatMessage(commonMessages.emailLabel)"
				/>
			</div>

			<div class="iconified-input">
				<label for="username" hidden>{{ formatMessage(commonMessages.usernameLabel) }}</label>
				<UserIcon />
				<input
					id="username"
					v-model="username"
					type="text"
					autocomplete="username"
					class="auth-form__input"
					:placeholder="formatMessage(commonMessages.usernameLabel)"
				/>
			</div>

			<div class="iconified-input">
				<label for="password" hidden>{{ formatMessage(commonMessages.passwordLabel) }}</label>
				<KeyIcon />
				<input
					id="password"
					v-model="password"
					class="auth-form__input"
					type="password"
					autocomplete="new-password"
					:placeholder="formatMessage(commonMessages.passwordLabel)"
				/>
			</div>

			<div class="iconified-input">
				<label for="confirm-password" hidden>{{
					formatMessage(commonMessages.passwordLabel)
				}}</label>
				<KeyIcon />
				<input
					id="confirm-password"
					v-model="confirmPassword"
					type="password"
					autocomplete="new-password"
					class="auth-form__input"
					:placeholder="formatMessage(commonMessages.confirmPasswordLabel)"
				/>
			</div>

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

			<HCaptcha v-if="globals?.captcha_enabled" ref="captcha" v-model="token" />

			<button
				class="btn btn-primary continue-btn centered-btn"
				:disabled="globals?.captcha_enabled ? !token : false"
				@click="createAccount"
			>
				{{ formatMessage(messages.createAccountButton) }} <RightArrowIcon />
			</button>

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
	Checkbox,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	IntlFormatted,
	useVIntl,
} from '@modrinth/ui'

import HCaptcha from '@/components/ui/HCaptcha.vue'
import { getAuthUrl } from '@/composables/auth.js'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'auth.sign-up.title',
		defaultMessage: 'Sign Up',
	},
	signUpWithTitle: {
		id: 'auth.sign-up.title.sign-up-with',
		defaultMessage: 'Sign up with',
	},
	createAccountTitle: {
		id: 'auth.sign-up.title.create-account',
		defaultMessage: 'Or create an account yourself',
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
})

useHead({
	title: () => `${formatMessage(messages.title)} - Modrinth`,
})

const auth = await useAuth()
const route = useNativeRoute()

const redirectTarget = route.query.redirect

if (auth.value.user) {
	await navigateTo('/dashboard')
}

const captcha = ref()

const { data: globals } = await useAsyncData('auth-globals', async () => {
	try {
		return await useBaseFetch('globals', { internal: true })
	} catch (err) {
		console.error('Error fetching globals:', err)
		return { captcha_enabled: true }
	}
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

		const res = await useBaseFetch('auth/create', {
			method: 'POST',
			body: {
				username: username.value,
				password: password.value,
				email: email.value,
				challenge: token.value,
				sign_up_newsletter: subscribe.value,
			},
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
