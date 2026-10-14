import { createContext } from '@modrinth/ui'

export type InstanceDefaultsTab = 'java' | 'window' | 'hooks'

/**
 * Opens the app settings modal from anywhere in the app, optionally on a specific tab.
 */
export interface SettingsActions {
	openSettings: () => void
	openProfile: () => void
	openFeatureFlags: () => void
	openSyncedOptions: () => void
	openInstanceDefaults: (tab: InstanceDefaultsTab) => void
}

export const [injectSettingsActions, provideSettingsActions] = createContext<SettingsActions>(
	'root',
	'settingsActions',
)
