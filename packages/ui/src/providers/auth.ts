import type { Labrinth } from '@modrinth/api-client'
import type { Ref } from 'vue'

import { createContext } from './create-context'

export type AuthUser = Labrinth.Users.v2.User | Labrinth.Users.v3.User

export interface AuthProvider {
	session_token: Ref<string | null>
	user: Ref<AuthUser | null>
	/** True once the initial auth check has completed (regardless of result). */
	isReady?: Ref<boolean>
	requestSignIn: (redirectPath: string) => void | Promise<void>
}

export const [injectAuth, provideAuth] = createContext<AuthProvider>('root', 'auth')
