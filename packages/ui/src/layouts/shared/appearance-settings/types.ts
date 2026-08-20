import type { Labrinth } from '@modrinth/api-client'
import type { ComputedRef, Ref } from 'vue'

export type AppearanceTheme = Labrinth.Users.v3.Theme
export type AppearanceThemeSelection = AppearanceTheme | 'system'
export type ProjectLayout = Labrinth.Users.v3.LayoutOption
export type SidebarPreferences = Labrinth.Users.v3.SidebarPreferences
export type AppearanceRef<T> = Ref<T> | ComputedRef<T>
export type AppearanceSetter<T> = (value: T) => void | Promise<void>

export const projectDisplayLocations = [
	'mod',
	'plugin',
	'resourcepack',
	'modpack',
	'shader',
	'datapack',
	'server',
	'user',
] as const

export type ProjectDisplayLocation = (typeof projectDisplayLocations)[number]

export function isProjectDisplayLocation(value: string): value is ProjectDisplayLocation {
	return projectDisplayLocations.some((location) => location === value)
}

export interface ProjectLayoutSetting {
	type: ProjectDisplayLocation
	layout: ProjectLayout
}

export interface WritableAppearanceSetting<T> {
	value: AppearanceRef<T>
	set: AppearanceSetter<T>
}
