export interface SettingDefinitionBase<T> {
	type: SettingDefinitionTypes
	id: string
	title: string
	description: string
	default: T
	onChange?(previous: T | undefined, current: T): void
}

export interface ToggleSettingDefinition extends SettingDefinitionBase<boolean> {
	type: 'toggle'
}

export interface EnumSettingDefinition<T extends string = string> extends SettingDefinitionBase<T> {
	type: 'enum'
	entries: {
		label: string
		value: T
	}[]
}

export interface StringSettingDefinition extends SettingDefinitionBase<string> {
	type: 'string'
	regex?: RegExp
}

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

export type SettingDefinitions = ReturnType<(typeof setting)[keyof typeof setting]>
export type SettingDefinitionTypes = SettingDefinitions['type']

export const setting = {
	asEnum: <const E extends ReadonlyArray<{ label: string; value: string }>>(
		data: EnumSettingDefinition<E[number]['value']> & { entries: E },
	): EnumSettingDefinition<E[number]['value']> => {
		return data as EnumSettingDefinition<E[number]['value']>
	},
	asToggle: (data: ToggleSettingDefinition) => data,
	asString: (data: StringSettingDefinition) => data,
	asColor: (data: ColorSettingDefinition) => data,
	asNumber: (data: NumberSettingDefinition) => data,
}

export function isValidFor(
	definitionBase: SettingDefinitionBase<unknown>,
	value: unknown,
): boolean {
	if (value != null) {
		// Tried my best with type safety but sadly having the `SettingDefinitions` as the type leads to issues handling types else where...
		const definition = definitionBase as SettingDefinitions
		if (definition.type == 'enum' && definition.entries.some((entry) => entry.value === value)) {
			return true
		} else if (definition.type == 'toggle' && typeof value === 'boolean') {
			return true
		} else if (definition.type === 'color' && typeof value === 'string') {
			return value === '' || /^#[0-9a-f]{6}$/i.test(value)
		} else if (definition.type === 'number' && typeof value === 'number') {
			return Number.isFinite(value) && value >= definition.min && value <= definition.max
		} else if (
			definition.type == 'string' &&
			typeof value === 'string' &&
			(!definition.regex || value.match(definition.regex))
		) {
			return true
		}
	}

	return false
}

export type SettingDefinition =
	| ToggleSettingDefinition
	| EnumSettingDefinition
	| StringSettingDefinition
	| ColorSettingDefinition
	| NumberSettingDefinition
