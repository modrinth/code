import type { Labrinth } from '@modrinth/api-client'
import { type AuthProvider, provideAuth } from '@modrinth/ui'
import { ref, watchEffect } from 'vue'

import { getSignInRedirectPath } from '~/composables/auth.js'

export function setupAuthProvider(auth: Awaited<ReturnType<typeof useAuth>>) {
	const router = useRouter()
	const route = useRoute()
	const sessionToken = ref<string | null>(null)
	const user = ref<Labrinth.Users.v2.User | null>(null)

	const authProvider: AuthProvider = {
		session_token: sessionToken,
		user,
		isReady: ref(true),
		requestSignIn: async (redirectPath: string) => {
			await router.push({
				path: '/auth/sign-in',
				query: {
					redirect: redirectPath || getSignInRedirectPath(route),
				},
			})
		},
	}

	watchEffect(() => {
		sessionToken.value = auth.value.token || null
		user.value = (auth.value.user as Labrinth.Users.v2.User | null) ?? null
	})

	provideAuth(authProvider)
}
