import type { Labrinth } from '@modrinth/api-client'
import { type AuthFlow, type AuthProvider, type AuthUser, provideAuth } from '@modrinth/ui'
import { computed, type Ref, ref, watchEffect } from 'vue'

type AppCredentials = {
	session?: string | null
	user?: Labrinth.Users.v2.User | null
}

export function setupAuthProvider(
	credentials: Ref<AppCredentials | null | undefined>,
	requestSignIn: (redirectPath: string, flow?: AuthFlow) => void | Promise<void>,
) {
	const sessionToken = ref<string | null>(null)
	const user = ref<AuthUser | null>(null)
	const isReady = computed(() => credentials.value !== undefined)

	const authProvider: AuthProvider = {
		session_token: sessionToken,
		user,
		isReady,
		requestSignIn,
	}

	watchEffect(() => {
		sessionToken.value = credentials.value?.session ?? null
		user.value = credentials.value?.user ?? null
	})

	provideAuth(authProvider)
}
