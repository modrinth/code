import type { AbstractModrinthClient } from '@modrinth/api-client'

import { createContext } from './index'

export const [injectModrinthClient, provideModrinthClient] = createContext<AbstractModrinthClient>(
	'root',
	'modrinthClient',
)
