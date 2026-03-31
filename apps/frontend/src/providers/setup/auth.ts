import type { Labrinth } from '@modrinth/api-client'
import { type AuthProvider, provideAuth } from '@modrinth/ui'
import { reactive, watchEffect } from 'vue'

export function setupAuthProvider(auth: Awaited<ReturnType<typeof useAuth>>) {
	const router = useRouter()
	const authProvider = reactive<AuthProvider>({
		session_token: null,
		user: null,
		requestSignIn: async (redirectPath: string) => {
			await router.push({
				path: '/auth/sign-in',
				query: {
					redirect: redirectPath,
				},
			})
		},
	})

	watchEffect(() => {
		authProvider.session_token = auth.value.token || null
		authProvider.user = (auth.value.user as Labrinth.Users.v2.User | null) ?? null
	})

	provideAuth(authProvider)
}
