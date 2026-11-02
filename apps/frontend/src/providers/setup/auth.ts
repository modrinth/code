import type { Labrinth } from '@modrinth/api-client'
import { type AuthFlow, type AuthProvider, type AuthUser, provideAuth } from '@modrinth/ui'
import { ref, watchEffect } from 'vue'

import { getSignInRedirectPath } from '~/composables/auth.ts'
import { promptAuth } from '~/composables/auth-modal.ts'

export function setupAuthProvider(auth: Awaited<ReturnType<typeof useAuth>>) {
	const route = useRoute()
	const sessionToken = ref<string | null>(null)
	const user = ref<AuthUser | null>(null)

	const authProvider: AuthProvider = {
		session_token: sessionToken,
		user,
		isReady: ref(true),
		requestSignIn: async (redirectPath: string, flow?: AuthFlow) => {
			await promptAuth({
				path: flow === 'sign-up' ? '/auth/sign-up' : '/auth/sign-in',
				query: {
					redirect: redirectPath || getSignInRedirectPath(route),
				},
			})
		},
	}

	watchEffect(() => {
		sessionToken.value = auth.value.token || null
		user.value = (auth.value.user as Labrinth.Users.v3.User | null) ?? null
	})

	provideAuth(authProvider)
	return authProvider
}
