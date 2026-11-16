import type { MaybeRefOrGetter } from 'vue'

import { createContext } from './create-context'

export interface AppBackupContext {
	createBackup: () => Promise<void>
	available?: MaybeRefOrGetter<boolean>
}

export const [injectAppBackup, provideAppBackup] = createContext<AppBackupContext>(
	'AppBackupContext',
	'appBackupContext',
)
