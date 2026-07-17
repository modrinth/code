export interface SettingDefinitionBase<T> {
	type: string,
	id: string,
	title: string,
	description: string,
	default: T,
	onChange?: (previous: T | undefined, current: T) => void,
}

export interface ToggleSettingDefinition extends SettingDefinitionBase<boolean> {
	type: 'toggle',
}

export interface EnumSettingDefinition extends SettingDefinitionBase<string> {
	type: 'enum',
	entries: {
		label: string,
		value: string,
	}[],
}

export interface StringSettingDefinition extends SettingDefinitionBase<string> {
	type: 'string',
	multiline?: boolean,
}

export type SettingDefinition = ToggleSettingDefinition | EnumSettingDefinition | StringSettingDefinition
