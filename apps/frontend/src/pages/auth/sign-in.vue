<template>
	<SignInView
		v-model:email="email"
		v-model:password="password"
		v-model:token="token"
		v-model:two-factor-code="twoFactorCode"
		:subtle-launcher-redirect-uri="subtleLauncherRedirectUri"
		:flow="flow"
		:redirect-target="redirectTarget"
		:route-query="route.query"
		:globals="globals"
		:on-password-sign-in="beginPasswordSignIn"
		:on-two-factor-sign-in="begin2FASignIn"
		:on-set-captcha-ref="setCaptchaRef"
	/>
</template>

<script setup>
import {
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'

import SignInView from '@/components/ui/auth/SignIn.vue'
import { getLauncherRedirectUrl } from '@/composables/auth.ts'

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

const redirectTarget = route.query.redirect || ''
const subtleLauncherRedirectUri = ref()

if (route.query.code) {
	await finishSignIn()
}

if (auth.value.user) {
	await finishSignIn()
}

const captcha = ref()
const setCaptchaRef = (captchaRef) => {
	captcha.value = captchaRef
}

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
const password = ref('')
const token = ref('')

const flow = ref(route.query.flow)

async function beginPasswordSignIn() {
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
		const res = await client.labrinth.auth_v2.login2FA({
			flow: flow.value,
			code: twoFactorCode.value ? twoFactorCode.value.toString() : twoFactorCode.value,
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
