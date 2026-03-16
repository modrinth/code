<template>
	<div v-if="subtleLauncherRedirectUri">
		<iframe
			:src="subtleLauncherRedirectUri"
			class="fixed left-0 top-0 z-[9999] m-0 h-full w-full border-0 p-0"
		></iframe>
	</div>
	<div v-else>
		<template v-if="flow && !subtleLauncherRedirectUri">
			<label for="two-factor-code">
				<span class="label__title">{{ formatMessage(messages.twoFactorCodeLabel) }}</span>
				<span class="label__description">
					{{ formatMessage(messages.twoFactorCodeLabelDescription) }}
				</span>
			</label>
			<StyledInput
				id="two-factor-code"
				v-model="twoFactorCode"
				:maxlength="11"
				inputmode="numeric"
				:placeholder="formatMessage(messages.twoFactorCodeInputPlaceholder)"
				autocomplete="one-time-code"
				@keyup.enter="begin2FASignIn"
			/>

			<button class="btn btn-primary continue-btn" @click="begin2FASignIn">
				{{ formatMessage(commonMessages.signInButton) }} <RightArrowIcon />
			</button>
		</template>
		<template v-else>
			<h1>{{ formatMessage(messages.signInWithLabel) }}</h1>

			<section class="third-party">
				<a class="btn" :href="getAuthUrl('discord', redirectTarget)">
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

			<h1>{{ formatMessage(messages.usePasswordLabel) }}</h1>

			<section class="auth-form">
				<label for="email" hidden>{{ formatMessage(commonMessages.emailUsernameLabel) }}</label>
				<StyledInput
					id="email"
					v-model="email"
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
					v-model="password"
					:icon="KeyIcon"
					type="password"
					autocomplete="current-password"
					:placeholder="formatMessage(commonMessages.passwordLabel)"
					wrapper-class="w-full"
				/>

				<HCaptcha v-if="globals?.captcha_enabled" ref="captcha" v-model="token" />

				<button
					class="btn btn-primary continue-btn centered-btn"
					:disabled="globals?.captcha_enabled ? !token : false"
					@click="beginPasswordSignIn()"
				>
					{{ formatMessage(commonMessages.signInButton) }} <RightArrowIcon />
				</button>

				<div class="auth-form__additional-options">
					<IntlFormatted :message-id="messages.additionalOptionsLabel">
						<template #forgot-password-link="{ children }">
							<NuxtLink
								class="text-link"
								:to="{
									path: '/auth/reset-password',
									query: route.query,
								}"
							>
								<component :is="() => children" />
							</NuxtLink>
						</template>
						<template #create-account-link="{ children }">
							<NuxtLink
								class="text-link"
								:to="{
									path: '/auth/sign-up',
									query: route.query,
								}"
							>
								<component :is="() => children" />
							</NuxtLink>
						</template>
					</IntlFormatted>
				</div>
			</section>
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
	commonMessages,
	defineMessages,
	injectNotificationManager,
	IntlFormatted,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'

import HCaptcha from '@/components/ui/HCaptcha.vue'
import { getAuthUrl, getLauncherRedirectUrl } from '@/composables/auth.js'

const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	additionalOptionsLabel: {
		id: 'auth.sign-in.additional-options',
		defaultMessage:
			'<forgot-password-link>Forgot password?</forgot-password-link> • <create-account-link>Create an account</create-account-link>',
	},
	signInWithLabel: {
		id: 'auth.sign-in.sign-in-with',
		defaultMessage: 'Sign in with',
	},
	signInTitle: {
		id: 'auth.sign-in.title',
		defaultMessage: 'Sign In',
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
	usePasswordLabel: {
		id: 'auth.sign-in.use-password',
		defaultMessage: 'Or use a password',
	},
})

useHead({
	title() {
		return `${formatMessage(messages.signInTitle)} - Modrinth`
	},
})

const auth = await useAuth()
const route = useNativeRoute()

const redirectTarget = route.query.redirect || ''
const subtleLauncherRedirectUri = ref()

if (route.query.code && !route.fullPath.includes('new_account=true')) {
	await finishSignIn()
}

if (auth.value.user) {
	await finishSignIn()
}

const captcha = ref()

const { data: globals } = useQuery({
	queryKey: ['auth-globals'],
	queryFn: async () => {
		try {
			return await useBaseFetch('globals', { internal: true })
		} catch (err) {
			console.error('Error fetching globals:', err)
			return { captcha_enabled: true }
		}
	},
})

const email = ref('')
const password = ref('')
const token = ref('')

const flow = ref(route.query.flow)

async function beginPasswordSignIn() {
	startLoading()
	try {
		const res = await useBaseFetch('auth/login', {
			method: 'POST',
			body: {
				username: email.value,
				password: password.value,
				challenge: token.value,
			},
		})

		if (res.flow) {
			flow.value = res.flow
		} else {
			await finishSignIn(res.session)
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

const twoFactorCode = ref(null)
async function begin2FASignIn() {
	startLoading()
	try {
		const res = await useBaseFetch('auth/login/2fa', {
			method: 'POST',
			body: {
				flow: flow.value,
				code: twoFactorCode.value ? twoFactorCode.value.toString() : twoFactorCode.value,
			},
		})

		await finishSignIn(res.session)
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

async function finishSignIn(token) {
	if (route.query.launcher) {
		if (!token) {
			token = auth.value.token
		}

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

	if (token) {
		await useAuth(token)
		await useUser()
		queryClient.clear()
	}

	if (route.query.redirect) {
		const redirect = decodeURIComponent(route.query.redirect)
		await navigateTo(redirect, {
			replace: true,
		})
	} else {
		await navigateTo('/dashboard')
	}
}
</script>
