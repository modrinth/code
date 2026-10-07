import { useAuthCookie } from '@/composables/auth-cookie.ts'

/**
 * Keeps the session token out of the SSR payload so cached or shared HTML never carries it.
 * The client restores it from the auth cookie, which every server-side token write also updates.
 */
export default defineNuxtPlugin({
	name: 'auth-token-payload',
	enforce: 'pre',
	setup(nuxtApp) {
		const auth = useAuthState()

		if (import.meta.server) {
			nuxtApp.hooks.hook('app:rendered', () => {
				auth.value = { ...auth.value, token: '' }
			})
			return
		}

		auth.value.token = useAuthCookie().value ?? ''
	},
})
