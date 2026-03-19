import { createContext } from '.'

export interface ImportableLauncher {
	name: string
	path: string
	instances: string[]
}

export interface InstanceImportProvider {
	/** Returns launchers with instances already populated (one round trip on mount) */
	getDetectedLaunchers: () => Promise<ImportableLauncher[]>
	/** Only needed for manually-added launcher paths */
	getImportableInstances: (launcherName: string, path: string) => Promise<string[]>
	/** Perform the actual import */
	importInstances: (
		selections: { launcher: string; path: string; instanceNames: string[] }[],
	) => Promise<void>
	/** Open a directory picker (platform-specific) */
	selectDirectory: () => Promise<string | null>
}

export const [injectInstanceImport, provideInstanceImport] =
	createContext<InstanceImportProvider>('InstanceImport')
