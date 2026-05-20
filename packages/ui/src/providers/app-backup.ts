import { createContext } from './create-context'

export interface AppBackupContext {
	createBackup: () => Promise<void>
}

export const [injectAppBackup, provideAppBackup] = createContext<AppBackupContext>(
	'AppBackupContext',
	'appBackupContext',
)
