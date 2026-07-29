import { promotePendingSignInOAuthProvider } from '@/composables/auth.ts'

export default defineNuxtPlugin(async () => {
	const auth = await useAuth()
	if (auth.value.user) {
		promotePendingSignInOAuthProvider()
	}
})
