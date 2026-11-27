import type {
	EditableGameSetting,
	GameOptionCanonicalValue,
	GameSettingsEditorState,
} from '@/helpers/game-options'

export type GameSettingEdit = {
	option_id: string
	sync_enabled?: boolean
	canonical_value?: GameOptionCanonicalValue | null
}

export function canonicalValuesEqual(
	left: GameOptionCanonicalValue | null | undefined,
	right: GameOptionCanonicalValue | null | undefined,
): boolean {
	return JSON.stringify(left ?? null) === JSON.stringify(right ?? null)
}

export function applyGameSettingEdits(
	state: GameSettingsEditorState,
	edits: readonly GameSettingEdit[],
): GameSettingsEditorState {
	if (edits.length === 0) return state

	const editsById = new Map<string, GameSettingEdit>()
	for (const edit of edits) {
		editsById.set(edit.option_id, { ...editsById.get(edit.option_id), ...edit })
	}
	return {
		...state,
		settings: state.settings.map((setting): EditableGameSetting => {
			const edit = editsById.get(setting.option_id)
			if (!edit) return setting
			return {
				...setting,
				...(edit.sync_enabled !== undefined ? { sync_enabled: edit.sync_enabled } : {}),
				...(edit.canonical_value !== undefined
					? {
							canonical_value: edit.canonical_value,
							value_state: edit.canonical_value ? 'canonical' : 'unset',
						}
					: {}),
				validation_error: null,
			}
		}),
	}
}

export function restoreGameSettingValues(
	state: GameSettingsEditorState,
	snapshot: readonly EditableGameSetting[],
): GameSettingsEditorState {
	const previousById = new Map(snapshot.map((setting) => [setting.option_id, setting]))
	return {
		...state,
		settings: state.settings.map((setting) => {
			const previous = previousById.get(setting.option_id)
			if (!previous) return setting
			return {
				...setting,
				sync_enabled: previous.sync_enabled,
				canonical_value: previous.canonical_value,
				value_state: previous.value_state,
				validation_error: previous.validation_error,
			}
		}),
	}
}

export function gameSettingNumberScale(setting: EditableGameSetting): number {
	if (setting.editor.unit !== 'percent') return 1
	return setting.option_id === 'sensitivity' ? 200 : 100
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
				? String(Number((Number(value.value) * gameSettingNumberScale(setting)).toFixed(8)))
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

export function isKeybindSetting(setting: EditableGameSetting): boolean {
	return (
		setting.editor.type === 'key_binding' ||
		(setting.editor.type === 'external_raw' && !!setting.raw_key?.startsWith('key_'))
	)
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
						? String(Number((parsed / gameSettingNumberScale(setting)).toFixed(8)))
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
