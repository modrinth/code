import { rememberStoredAccount } from '@/composables/accounts.ts'
import { promotePendingSignInOAuthProvider } from '@/composables/auth.ts'

export default defineNuxtPlugin(async () => {
	const auth = await useAuth()
	if (!auth.value.user || !auth.value.token) {
		return
	}

	rememberStoredAccount(auth.value.user, auth.value.token)
	promotePendingSignInOAuthProvider()
})
