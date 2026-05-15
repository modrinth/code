import type { AbstractModrinthClient } from '@icarus/api-client'

import { createContext } from './create-context'

export const [injectIcarusClient, provideIcarusClient] = createContext<AbstractModrinthClient>(
	'root',
	'modrinthClient',
)
