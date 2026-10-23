import { useLocalStorage } from '@vueuse/core'
import { type MaybeRefOrGetter, type Ref, toValue } from 'vue'

import { isValidSettingValue, type SettingDefinition, type SettingDefinitionBase } from './types'

/** Where settings' values are read from and written to; reads are reactive. */
export interface SettingsStore {
	get<T>(definition: SettingDefinitionBase<T>): T
	set<T>(definition: SettingDefinitionBase<T>, value: T): void
}

export interface LocalSettingsOptions {
	/** Prepended to each setting's id to make its local storage key, unless it has its own. */
	prefix?: string
	/** Reads the stored values once mounted, so server rendered pages hydrate with the defaults. */
	initOnMounted?: boolean
}

/**
 * Settings kept in local storage, one key per setting. Other stores over the same keys (e.g. a
 * settings modal and the layout the settings apply to) follow each other's changes. Only values
 * the user picked are stored, so a setting left alone follows its default if that changes.
 */
export function useLocalSettings(
	definitions: readonly SettingDefinition[],
	options: LocalSettingsOptions = {},
): SettingsStore {
	const values = new Map<string, Ref<unknown>>()
	for (const definition of definitions) {
		values.set(
			definition.id,
			useLocalStorage<unknown>(
				definition.storageKey ?? `${options.prefix ?? ''}${definition.id}`,
				definition.default,
				{ initOnMounted: options.initOnMounted, writeDefaults: false },
			),
		)
	}

	return {
		get<T>(definition: SettingDefinitionBase<T>): T {
			const value = values.get(definition.id)?.value
			return isValidSettingValue(definition, value) ? (value as T) : definition.default
		},
		set<T>(definition: SettingDefinitionBase<T>, value: T) {
			const stored = values.get(definition.id)
			if (!stored) return
			const previous = stored.value as T | undefined
			stored.value = value
			definition.onChange?.(previous, value)
		},
	}
}

/**
 * Settings kept in a reactive record of values by setting id, e.g. one loaded from and saved to a
 * cookie by the host. `onWrite` runs after each change, to persist the record.
 */
export function recordSettings(
	record: MaybeRefOrGetter<Record<string, unknown>>,
	onWrite?: () => void,
): SettingsStore {
	return {
		get<T>(definition: SettingDefinitionBase<T>): T {
			const value = toValue(record)[definition.id]
			return isValidSettingValue(definition, value as T) ? (value as T) : definition.default
		},
		set<T>(definition: SettingDefinitionBase<T>, value: T) {
			const values = toValue(record)
			const previous = values[definition.id] as T | undefined
			values[definition.id] = value
			definition.onChange?.(previous, value)
			onWrite?.()
		},
	}
}

/** The settings of every group, e.g. to make a store holding all of them. */
export function settingsOf(groups: readonly { settings: SettingDefinition[] }[]) {
	return groups.flatMap((group) => group.settings)
}
