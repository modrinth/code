import type { Labrinth } from '@icarus/api-client'
import type { Ref } from 'vue'

import { createContext } from '.'

export interface UserPageContext {
	user: Ref<Labrinth.Users.v3.User>
}

export const [injectUserPageContext, provideUserPageContext] = createContext<UserPageContext>(
	'root',
	'userPageContext',
)
