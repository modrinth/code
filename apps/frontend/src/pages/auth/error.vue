<script setup lang="ts">
import { commonMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'

const route = useRoute()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const getQueryValue = (value: unknown): string | undefined => {
	if (Array.isArray(value)) {
		return typeof value[0] === 'string' ? value[0] : undefined
	}

	return typeof value === 'string' ? value : undefined
}

const message = getQueryValue(route.query.message) || 'Authentication failed'
const redirect = getQueryValue(route.query.redirect)

const getRedirectPath = () => {
	if (!import.meta.client || !redirect) {
		return '/auth/sign-in'
	}

	try {
		const url = new URL(redirect, window.location.origin)

		if (url.origin !== window.location.origin || url.pathname === '/auth/error') {
			return '/auth/sign-in'
		}

		return `${url.pathname}${url.search}${url.hash}`
	} catch {
		return '/auth/sign-in'
	}
}

definePageMeta({
	layout: 'empty',
})

onMounted(async () => {
	addNotification({
		title: formatMessage(commonMessages.errorNotificationTitle),
		text: message,
		type: 'error',
	})

	await navigateTo(getRedirectPath(), {
		replace: true,
	})
})
</script>
