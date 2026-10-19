import type { Component } from 'vue'

import type { MessageDescriptor } from '../../composables/i18n'
import type { SettingsStore } from './store'

/** Text shown for a setting: a message to translate, or text that's already final. */
export type SettingText = string | MessageDescriptor

export interface SettingDefinitionBase<T> {
	type: SettingDefinitionType
	id: string
	title: SettingText
	description?: SettingText
	default: T
	/**
	 * Where the value is kept by stores backed by local storage, in place of the store's prefix
	 * and the id, e.g. to keep reading a value saved before the setting was defined.
	 */
	storageKey?: string
	/** Declared as a method so settings with narrower values (e.g. an enum's) fit wider types. */
	onChange?(previous: T | undefined, current: T): void
}

export interface ToggleSettingDefinition extends SettingDefinitionBase<boolean> {
	type: 'toggle'
}

export interface EnumSettingEntry<T extends string = string> {
	value: T
	label: SettingText
	icon?: Component
}

export interface EnumSettingDefinition<T extends string = string> extends SettingDefinitionBase<T> {
	type: 'enum'
	entries: readonly EnumSettingEntry<T>[]
	/** `chips` lays a handful of short entries out side by side; `select` (the default) is a dropdown. */
	display?: 'chips' | 'select'
}

export interface StringSettingDefinition extends SettingDefinitionBase<string> {
	type: 'string'
	regex?: RegExp
	placeholder?: SettingText
}

/** A `#rrggbb` color, or an empty string for none. */
export interface ColorSettingDefinition extends SettingDefinitionBase<string> {
	type: 'color'
}

export interface NumberSettingDefinition extends SettingDefinitionBase<number> {
	type: 'number'
	min: number
	max: number
	step: number
	unit?: string
}

export type SettingDefinition =
	| ToggleSettingDefinition
	| EnumSettingDefinition
	| StringSettingDefinition
	| ColorSettingDefinition
	| NumberSettingDefinition

export type SettingDefinitionType = SettingDefinition['type']

/** Settings shown together under a heading. */
export interface SettingGroup {
	id: string
	title: SettingText
	description?: SettingText
	settings: SettingDefinition[]
	/** Where the group's settings are kept, when it isn't the panel's store. */
	store?: SettingsStore
}

/** Builders that keep each kind of setting's value typed, e.g. an enum's to its entries. */
export const defineSetting = {
	toggle: (definition: Omit<ToggleSettingDefinition, 'type'>): ToggleSettingDefinition => ({
		...definition,
		type: 'toggle',
	}),
	enum: <const T extends string>(
		definition: Omit<EnumSettingDefinition<T>, 'type'>,
	): EnumSettingDefinition<T> => ({ ...definition, type: 'enum' }),
	string: (definition: Omit<StringSettingDefinition, 'type'>): StringSettingDefinition => ({
		...definition,
		type: 'string',
	}),
	color: (definition: Omit<ColorSettingDefinition, 'type'>): ColorSettingDefinition => ({
		...definition,
		type: 'color',
	}),
	number: (definition: Omit<NumberSettingDefinition, 'type'>): NumberSettingDefinition => ({
		...definition,
		type: 'number',
	}),
}

/** Whether a stored value is one the setting can take, e.g. still one of an enum's entries. */
export function isValidSettingValue<T>(definition: SettingDefinitionBase<T>, value: T) {
	if (value == null) return false
	const typed = definition as SettingDefinition
	switch (typed.type) {
		case 'toggle':
			return typeof value === 'boolean'
		case 'enum':
			return typed.entries.some((entry) => entry.value === value)
		case 'string':
			return typeof value === 'string' && (!typed.regex || typed.regex.test(value))
		case 'color':
			return typeof value === 'string' && (value === '' || /^#[0-9a-f]{6}$/i.test(value))
		case 'number':
			return (
				typeof value === 'number' &&
				Number.isFinite(value) &&
				value >= typed.min &&
				value <= typed.max
			)
		default:
			return true
	}
}
