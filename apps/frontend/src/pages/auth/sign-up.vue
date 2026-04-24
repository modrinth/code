<template>
	<SignUpView
		v-if="!isCreateAccountStep"
		v-model:email="email"
		v-model:password="password"
		:redirect-target="redirectTarget"
		:show-other-options="showOtherOptions"
		:route-query="route.query"
		:on-toggle-other-options="toggleOtherOptions"
		:on-continue-with-email="continueWithEmail"
	/>
	<CreateAccountView
		v-else
		v-model:date-of-birth="dateOfBirth"
		v-model:username="username"
		v-model:token="token"
		v-model:subscribe="subscribe"
		:globals="globals"
		:on-complete-sign-up="createAccount"
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
import { useStorage } from '@vueuse/core'

import CreateAccountView from '@/components/ui/auth/CreateAccount.vue'
import SignUpView from '@/components/ui/auth/SignUp.vue'
import {
	LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
	promotePendingSignInOAuthProvider,
} from '@/composables/auth.ts'

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'auth.sign-up.title',
		defaultMessage: 'Sign Up',
	},
	ageRequirementWarningTitle: {
		id: 'auth.sign-up.age-requirement.warning-title',
		defaultMessage: 'Age requirement',
	},
	under13HelperText: {
		id: 'auth.create-account.date-of-birth.under13-helper',
		defaultMessage: 'You cannot create an account at Modrinth unless you are over 13 years old.',
	},
})

useHead({
	title: () => `${formatMessage(messages.title)} - Modrinth`,
})

const auth = await useAuth()
const route = useNativeRoute()
const pendingSignInOAuthProvider = useStorage(PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY, null)
const lastSignInOAuthProvider = useStorage(LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY, null)

const redirectTarget = route.query.redirect
const showOtherOptions = ref(false)
const isCreateAccountStep = ref(false)

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
const dateOfBirth = ref('')
const username = ref('')
const token = ref('')
const subscribe = ref(false)
const USERNAME_MAX_LENGTH = 39
const MAX_USERNAME_SUFFIX_RETRIES = 10
const MAX_RANDOM_USERNAME_ATTEMPTS = 10

async function continueWithEmail() {
	pendingSignInOAuthProvider.value = null
	lastSignInOAuthProvider.value = null
	startLoading()
	try {
		const generatedUsername = await findAvailableGeneratedUsername()

		token.value = ''
		username.value = generatedUsername
		isCreateAccountStep.value = true
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}

function generateUsernameFromEmail(emailAddress) {
	const [localPart = ''] = emailAddress.trim().toLowerCase().split('@')
	const sanitized = localPart
		.replace(/[^a-zA-Z0-9_-]/g, '_')
		.replace(/_+/g, '_')
		.replace(/^_+|_+$/g, '')

	return (sanitized || 'user').slice(0, USERNAME_MAX_LENGTH)
}

async function findAvailableGeneratedUsername() {
	const baseUsername = generateUsernameFromEmail(email.value)

	for (let suffixAttempt = 0; suffixAttempt <= MAX_USERNAME_SUFFIX_RETRIES; suffixAttempt++) {
		const candidateUsername = appendUsernameSuffix(baseUsername, suffixAttempt)

		try {
			await validateCreateAccountCandidate(candidateUsername)
			return candidateUsername
		} catch (err) {
			if (!isUsernameTakenValidationError(err)) {
				throw err
			}
		}
	}

	return await findAvailableRandomUsername()
}

function appendUsernameSuffix(baseUsername, suffixAttempt) {
	if (suffixAttempt === 0) {
		return baseUsername
	}

	const suffix = String(suffixAttempt)
	const maxBaseLength = Math.max(1, USERNAME_MAX_LENGTH - suffix.length)
	return `${baseUsername.slice(0, maxBaseLength)}${suffix}`
}

function generateRandomUsername() {
	const randomSuffix = `${Date.now().toString(36)}${Math.random().toString(36).slice(2, 8)}`
	const base = `user_${randomSuffix}`
	return base.slice(0, USERNAME_MAX_LENGTH)
}

async function findAvailableRandomUsername() {
	for (let attempt = 0; attempt < MAX_RANDOM_USERNAME_ATTEMPTS; attempt++) {
		const candidateUsername = generateRandomUsername()

		try {
			await validateCreateAccountCandidate(candidateUsername)
			return candidateUsername
		} catch (err) {
			if (!isUsernameTakenValidationError(err)) {
				throw err
			}
		}
	}

	throw new Error('Unable to find an available username. Please choose one manually.')
}

async function validateCreateAccountCandidate(candidateUsername) {
	await client.labrinth.auth_v2.validateCreateAccount({
		username: candidateUsername,
		password: password.value,
		email: email.value,
	})
}

function isUsernameTakenValidationError(error) {
	const errorCode = error?.data?.error ?? error?.v1Error?.error ?? error?.responseData?.error
	return errorCode === 'username_taken'
}

async function createAccount() {
	startLoading()
	try {
		const res = await client.labrinth.auth_v2.createAccount({
			username: username.value.trim() || generateUsernameFromEmail(email.value),
			password: password.value,
			email: email.value,
			challenge: token.value,
			sign_up_newsletter: subscribe.value,
		})

		await useAuth(res.session)
		await useUser()

		promotePendingSignInOAuthProvider()

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
