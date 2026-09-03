import { toRaw } from 'vue'

import type {
	EditableGameSetting,
	GameOptionCanonicalValue,
	GameSettingChange,
	GameSettingsEditorState,
} from '@/helpers/game-options'

function clonePlainValue(value: unknown): unknown {
	if (Array.isArray(value)) {
		return toRaw(value).map(clonePlainValue)
	}
	if (value !== null && typeof value === 'object') {
		return Object.fromEntries(
			Object.entries(toRaw(value)).map(([key, nestedValue]) => [key, clonePlainValue(nestedValue)]),
		)
	}
	return value
}

export function cloneGameSettingsState(state: GameSettingsEditorState): GameSettingsEditorState {
	return clonePlainValue(state) as GameSettingsEditorState
}

export function canonicalValuesEqual(
	left: GameOptionCanonicalValue | null | undefined,
	right: GameOptionCanonicalValue | null | undefined,
): boolean {
	return JSON.stringify(left ?? null) === JSON.stringify(right ?? null)
}

export function gameSettingChanges(
	base: GameSettingsEditorState | null,
	draft: GameSettingsEditorState | null,
	forcedValueOptionIds: ReadonlySet<string> = new Set(),
): GameSettingChange[] {
	if (!base || !draft) return []

	const baseSettings = new Map(base.settings.map((setting) => [setting.option_id, setting]))
	return draft.settings.flatMap((setting) => {
		const previous = baseSettings.get(setting.option_id)
		if (!previous) return []

		const syncChanged = previous.sync_enabled !== setting.sync_enabled
		const valueChanged = !canonicalValuesEqual(previous.canonical_value, setting.canonical_value)
		const promoteLocalValue =
			forcedValueOptionIds.has(setting.option_id) &&
			previous.value_state !== 'canonical' &&
			setting.canonical_value !== null &&
			setting.canonical_value !== undefined
		if (!syncChanged && !valueChanged && !promoteLocalValue) return []

		return [
			{
				option_id: setting.option_id,
				base_option_revision: previous.option_revision,
				...(syncChanged ? { sync_enabled: setting.sync_enabled } : {}),
				...(valueChanged || promoteLocalValue
					? { canonical_value: setting.canonical_value ?? null }
					: {}),
			},
		]
	})
}

export function canonicalValueText(setting: EditableGameSetting): string {
	const value = setting.canonical_value
	if (!value) return ''

	switch (value.type) {
		case 'bool':
			return value.value ? 'true' : 'false'
		case 'integer':
		case 'decimal':
			return setting.editor.unit === 'percent'
				? String(Number((Number(value.value) * 100).toFixed(8)))
				: String(value.value)
		case 'string_list':
			return value.value.join(', ')
		default:
			return value.value
	}
}

export function canonicalBooleanValue(setting: EditableGameSetting): boolean | undefined {
	return setting.canonical_value?.type === 'bool' ? setting.canonical_value.value : undefined
}

export function canonicalValueFromInput(
	setting: EditableGameSetting,
	value: string | number | boolean | undefined,
): GameOptionCanonicalValue | null {
	if (value === undefined || value === '') return null

	switch (setting.editor.type) {
		case 'bool':
			return { type: 'bool', value: Boolean(value) }
		case 'integer': {
			const parsed = Number(value)
			return Number.isSafeInteger(parsed) ? { type: 'integer', value: parsed } : null
		}
		case 'decimal': {
			const parsed = Number(value)
			if (!Number.isFinite(parsed)) return null
			return {
				type: 'decimal',
				value:
					setting.editor.unit === 'percent'
						? String(Number((parsed / 100).toFixed(8)))
						: String(value),
			}
		}
		case 'enum':
			return { type: 'enum', value: String(value) }
		case 'string_list':
			return {
				type: 'string_list',
				value: String(value)
					.split(',')
					.map((item) => item.trim())
					.filter(Boolean),
			}
		case 'key_binding':
			return { type: 'key_binding', value: String(value) }
		case 'external_raw':
			return { type: 'external_raw', value: String(value) }
		default:
			return { type: 'text', value: String(value) }
	}
}

export function settingSearchText(
	setting: EditableGameSetting,
	label: string,
	description: string,
): string {
	return [label, description, setting.option_id, setting.raw_key]
		.filter(Boolean)
		.join(' ')
		.toLocaleLowerCase()
}

export function settingCanBeEnabled(setting: EditableGameSetting): boolean {
	return (
		!setting.controlled &&
		!setting.validation_error &&
		!['mixed', 'unset', 'invalid'].includes(setting.value_state) &&
		(setting.compatibility.total_participating === 0 || setting.compatibility.will_receive > 0)
	)
}
