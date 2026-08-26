import type { Labrinth } from '@modrinth/api-client'
import { toValue } from 'vue'

import { createContext } from '#ui/providers/create-context'

import type {
	AppearanceRef,
	AppearanceSetter,
	AppearanceTheme,
	AppearanceThemeSelection,
	ProjectDisplayLocation,
	ProjectLayout,
	ProjectLayoutSetting,
	SidebarPreferences,
	WritableAppearanceSetting,
} from '../types'

interface AppearanceSetting<T> {
	value: AppearanceRef<T>
	update: AppearanceSetter<T>
	disabled?: AppearanceRef<boolean>
}

interface ThemeSettings {
	current: AppearanceRef<AppearanceThemeSelection>
	options: AppearanceRef<readonly AppearanceThemeSelection[]>
	system: AppearanceRef<AppearanceTheme>
	update: AppearanceSetter<AppearanceThemeSelection>
	syncAcrossDevices: AppearanceSetting<boolean>
}

interface ProjectLayoutSettings {
	value: AppearanceRef<readonly ProjectLayoutSetting[]>
	update: (type: ProjectDisplayLocation, layout: ProjectLayout) => Promise<void>
}

interface SidebarSettings {
	value: AppearanceRef<SidebarPreferences>
	update: (key: keyof SidebarPreferences, enabled: boolean) => Promise<void>
}

export interface AppearanceSettingsContext {
	theme: ThemeSettings
	advancedRendering: AppearanceSetting<boolean>
	nativeDecorations?: AppearanceSetting<boolean>
	projectLayouts?: ProjectLayoutSettings
	externalLinksNewTab?: AppearanceSetting<boolean>
	sidebarPreferences?: SidebarSettings
}

export interface AppearanceSettingsProviderOptions {
	deferPersistence?: boolean
	theme: {
		current: AppearanceRef<AppearanceThemeSelection>
		options: AppearanceRef<readonly AppearanceThemeSelection[]>
		system: AppearanceRef<AppearanceTheme>
		set: AppearanceSetter<AppearanceThemeSelection>
		syncAcrossDevices: WritableAppearanceSetting<boolean>
		syncDisabled: AppearanceRef<boolean>
	}
	advancedRendering: WritableAppearanceSetting<boolean>
	nativeDecorations?: WritableAppearanceSetting<boolean>
	projectLayouts?: {
		value: AppearanceRef<readonly ProjectLayoutSetting[]>
		set: (type: ProjectDisplayLocation, layout: ProjectLayout) => void | Promise<void>
	}
	externalLinksNewTab?: WritableAppearanceSetting<boolean>
	sidebarPreferences?: {
		value: AppearanceRef<SidebarPreferences>
		set: (key: keyof SidebarPreferences, enabled: boolean) => void | Promise<void>
	}
	updatePreferences: (
		preferences: Labrinth.Users.v3.PartialUserPreferences,
	) => Promise<Labrinth.Users.v3.UserPreferences | undefined>
}

const layoutPreferenceKeys: Record<
	ProjectDisplayLocation,
	keyof Labrinth.Users.v3.LayoutPreferences
> = {
	mod: 'mods',
	plugin: 'plugins',
	datapack: 'datapacks',
	shader: 'shaders',
	resourcepack: 'resourcepacks',
	modpack: 'modpacks',
	server: 'servers',
	user: 'users',
}

const [injectAppearanceSettings, provideAppearanceSettingsContext] =
	createContext<AppearanceSettingsContext>('AppearanceSettingsLayout', 'appearanceSettings')

export { injectAppearanceSettings }

function createSetting<T>(setting: WritableAppearanceSetting<T>): AppearanceSetting<T> {
	return {
		value: setting.value,
		update: setting.set,
	}
}

export function provideAppearanceSettings(
	options: AppearanceSettingsProviderOptions,
): AppearanceSettingsContext {
	const projectLayouts = options.projectLayouts
	const sidebarPreferences = options.sidebarPreferences

	async function syncThemePreference(theme: AppearanceThemeSelection): Promise<void> {
		await options.updatePreferences({
			appearance:
				theme === 'system'
					? { auto: true }
					: {
							auto: false,
							theme,
						},
		})
	}

	async function syncThemeOrDisable(theme: AppearanceThemeSelection): Promise<void> {
		try {
			await syncThemePreference(theme)
		} catch {
			await options.theme.syncAcrossDevices.set(false)
		}
	}

	async function updateTheme(theme: AppearanceThemeSelection): Promise<void> {
		await options.theme.set(theme)
		if (options.deferPersistence) return
		if (!toValue(options.theme.syncAcrossDevices.value) || toValue(options.theme.syncDisabled)) {
			return
		}

		await syncThemeOrDisable(theme)
	}

	async function updateThemeSync(enabled: boolean): Promise<void> {
		if (toValue(options.theme.syncDisabled)) return

		await options.theme.syncAcrossDevices.set(enabled)
		if (options.deferPersistence) return
		if (enabled) {
			await syncThemeOrDisable(toValue(options.theme.current))
		}
	}

	async function updateProjectLayout(
		type: ProjectDisplayLocation,
		layout: ProjectLayout,
	): Promise<void> {
		if (!projectLayouts) return

		const previousLayout = toValue(projectLayouts.value).find(
			(setting) => setting.type === type,
		)?.layout

		await projectLayouts.set(type, layout)
		if (options.deferPersistence) return
		try {
			const layouts: Partial<Labrinth.Users.v3.LayoutPreferences> = {}
			layouts[layoutPreferenceKeys[type]] = layout
			await options.updatePreferences({ layouts })
		} catch {
			const currentLayout = toValue(projectLayouts.value).find(
				(setting) => setting.type === type,
			)?.layout
			if (previousLayout && currentLayout === layout) {
				await projectLayouts.set(type, previousLayout)
			}
		}
	}

	async function updateSidebarPreference(
		key: keyof SidebarPreferences,
		enabled: boolean,
	): Promise<void> {
		if (!sidebarPreferences) return

		const previousValue = toValue(sidebarPreferences.value)[key]
		await sidebarPreferences.set(key, enabled)
		if (options.deferPersistence) return
		try {
			const sidebars: Partial<SidebarPreferences> = {}
			sidebars[key] = enabled
			await options.updatePreferences({ sidebars })
		} catch {
			if (toValue(sidebarPreferences.value)[key] === enabled) {
				await sidebarPreferences.set(key, previousValue)
			}
		}
	}

	const context: AppearanceSettingsContext = {
		theme: {
			current: options.theme.current,
			options: options.theme.options,
			system: options.theme.system,
			update: updateTheme,
			syncAcrossDevices: {
				value: options.theme.syncAcrossDevices.value,
				disabled: options.theme.syncDisabled,
				update: updateThemeSync,
			},
		},
		advancedRendering: createSetting(options.advancedRendering),
		nativeDecorations: options.nativeDecorations
			? createSetting(options.nativeDecorations)
			: undefined,
		projectLayouts: projectLayouts
			? {
					value: projectLayouts.value,
					update: updateProjectLayout,
				}
			: undefined,
		externalLinksNewTab: options.externalLinksNewTab
			? createSetting(options.externalLinksNewTab)
			: undefined,
		sidebarPreferences: sidebarPreferences
			? {
					value: sidebarPreferences.value,
					update: updateSidebarPreference,
				}
			: undefined,
	}

	return provideAppearanceSettingsContext(context)
}
