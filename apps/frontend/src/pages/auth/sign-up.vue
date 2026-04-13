<template>
	<SignUpView
		v-model:email="email"
		v-model:password="password"
		v-model:token="token"
		v-model:subscribe="subscribe"
		:redirect-target="redirectTarget"
		:show-other-options="showOtherOptions"
		:route-query="route.query"
		:globals="globals"
		:on-toggle-other-options="toggleOtherOptions"
		:on-create-account="createAccount"
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
import { useQuery } from '@tanstack/vue-query'

import SignUpView from '@/components/ui/auth/SignUp.vue'

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'auth.sign-up.title',
		defaultMessage: 'Sign Up',
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
const setCaptchaRef = (captchaRef) => {
	captcha.value = captchaRef
}
const toggleOtherOptions = () => {
	showOtherOptions.value = !showOtherOptions.value
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
const subscribe = ref(false)

function generateUsernameFromEmail(emailAddress) {
	const [localPart = '', domainPart = ''] = emailAddress.trim().toLowerCase().split('@')
	const sanitized = `${localPart}_${domainPart}`
		.replace(/[^a-zA-Z0-9_-]/g, '_')
		.replace(/_+/g, '_')
		.replace(/^_+|_+$/g, '')

	return (sanitized || 'user').slice(0, 39)
}

async function createAccount() {
	startLoading()
	try {
		const res = await client.labrinth.auth_v2.createAccount({
			username: generateUsernameFromEmail(email.value),
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
