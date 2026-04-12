import type { Labrinth } from '@modrinth/api-client/src/modules/labrinth/types'
import type { Ref } from 'vue'

import { createContext } from './create-context'

export interface AuthProvider {
	session_token: Ref<string | null>
	user: Ref<Labrinth.Users.v2.User | null>
	/** True once the initial auth check has completed (regardless of result). */
	isReady?: Ref<boolean>
	requestSignIn: (redirectPath: string) => void | Promise<void>
}

export const [injectAuth, provideAuth] = createContext<AuthProvider>('root', 'auth')
