import type {
	ColorSettingDefinition,
	EnumSettingDefinition,
	NumberSettingDefinition,
	StringSettingDefinition,
	ToggleSettingDefinition,
} from '@modrinth/ui'

export type {
	ColorSettingDefinition,
	EnumSettingDefinition,
	NumberSettingDefinition,
	SettingDefinition,
	SettingDefinitionBase,
	StringSettingDefinition,
	ToggleSettingDefinition,
} from '@modrinth/ui'

/** Builders for the moderation settings, typed by the shared settings definitions. */
export const setting = {
	asEnum: <const E extends ReadonlyArray<{ label: string; value: string }>>(
		data: EnumSettingDefinition<E[number]['value']> & { entries: E },
	): EnumSettingDefinition<E[number]['value']> => data,
	asToggle: (data: ToggleSettingDefinition) => data,
	asString: (data: StringSettingDefinition) => data,
	asColor: (data: ColorSettingDefinition) => data,
	asNumber: (data: NumberSettingDefinition) => data,
}
