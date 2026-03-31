import type { Labrinth } from '@modrinth/api-client/src/modules/labrinth/types'

import { createContext } from './create-context'

export interface AuthProvider {
	session_token: string | null
	user: Labrinth.Users.v2.User | null
	requestSignIn: (redirectPath: string) => void | Promise<void>
}

export const [injectAuth, provideAuth] = createContext<AuthProvider>('root', 'auth')
