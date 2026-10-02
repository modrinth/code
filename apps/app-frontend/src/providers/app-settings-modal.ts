import type { InjectionKey } from 'vue'

export type AppSettingsDefaultsTab = 'sandbox' | 'java' | 'window' | 'hooks'

export type UnsavedChangesController = {
	hasChanges: () => boolean
	getOriginal: () => Record<string, unknown>
	getModified: () => Record<string, unknown>
	isSaving: () => boolean
	reset: () => void
	save: () => void | Promise<void>
}

export type AppSettingsModalContext = {
	close: () => boolean
	registerUnsavedChangesController: (controller: UnsavedChangesController | null) => void
}

export const appSettingsModalContextKey: InjectionKey<AppSettingsModalContext> =
	Symbol('appSettingsModalContext')
export const appSettingsModalOpenProfileKey: InjectionKey<() => void> = Symbol(
	'appSettingsModalOpenProfile',
)
export const appSettingsModalOpenDefaultsKey: InjectionKey<
	(tab: AppSettingsDefaultsTab) => void
> = Symbol('appSettingsModalOpenDefaults')
export const appSettingsModalOpenSyncedOptionsKey: InjectionKey<() => void> = Symbol(
	'appSettingsModalOpenSyncedOptions',
)
