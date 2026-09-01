import { invoke } from '@tauri-apps/api/core'

export type GameOptionCanonicalValue =
	| { type: 'bool'; value: boolean }
	| { type: 'integer'; value: number }
	| { type: 'decimal'; value: string }
	| { type: 'enum'; value: string }
	| { type: 'text'; value: string }
	| { type: 'string_list'; value: string[] }
	| { type: 'key_binding'; value: string }
	| { type: 'external_raw'; value: string }

export type GameOptionValueState = 'canonical' | 'uniform_local' | 'mixed' | 'unset' | 'invalid'

export type GameOptionCompatibilityStatus =
	| 'ready'
	| 'deferred'
	| 'waiting_for_file'
	| 'waiting_for_base'
	| 'not_available'
	| 'unsupported_value'
	| 'unmappable'
	| 'catalog_uncovered'
	| 'controlled'
	| 'degraded'

export type GameOptionMappingKind = 'direct' | 'legacy' | 'migrated' | 'lossless' | 'lossy'

export type GameOptionCompatibilityReason =
	| 'launcher_controlled'
	| 'catalog_uncovered'
	| 'inspection_failed'
	| 'waiting_for_options_file'
	| 'unsupported_value'
	| 'migrates_on_write'
	| 'waiting_for_compatible_base'
	| 'missing_setting'

export type GameOptionCompatibilityBucket = {
	instance_count: number
	write_keys: string[]
	eventual_keys: string[]
	game_versions: string[]
	status: GameOptionCompatibilityStatus
	mapping?: GameOptionMappingKind | null
	reason?: GameOptionCompatibilityReason | null
}

export type GameOptionCompatibilitySummary = {
	total_participating: number
	will_receive: number
	write_now: number
	left_local: number
	buckets: GameOptionCompatibilityBucket[]
}

export type GameOptionEditorChoice = {
	value: string
}

export type GameOptionEditor = {
	type:
		| 'bool'
		| 'integer'
		| 'decimal'
		| 'enum'
		| 'text'
		| 'string_list'
		| 'key_binding'
		| 'external_raw'
	min?: number | null
	max?: number | null
	step?: number | null
	unit?: string | null
	choices?: GameOptionEditorChoice[]
}

export type GameSettingCategory = {
	id: string
	is_custom?: boolean
}

export type GameOptionValidationError =
	| 'local_value_needs_saving'
	| 'missing_value'
	| 'no_compatible_instances'
	| 'invalid_value'
	| 'changed_since_opened'

export type EditableGameSetting = {
	option_id: string
	category_id: string
	kind: 'vanilla' | 'external'
	raw_key?: string | null
	sync_enabled: boolean
	canonical_value?: GameOptionCanonicalValue | null
	value_state: GameOptionValueState
	option_revision: number
	editor: GameOptionEditor
	compatibility: GameOptionCompatibilitySummary
	validation_error?: GameOptionValidationError | null
	controlled?: boolean
}

export type GameSettingsEditorState = {
	summary_revision: string
	canonical_revision: number
	catalog_revision: number
	total_participating: number
	categories: GameSettingCategory[]
	settings: EditableGameSetting[]
}

export type GameSettingChange = {
	option_id: string
	base_option_revision: number
	sync_enabled?: boolean
	canonical_value?: GameOptionCanonicalValue | null
}

export type UpdateGameSettingsRequest = {
	expected_summary_revision: string
	expected_canonical_revision: number
	expected_catalog_revision: number
	changes: GameSettingChange[]
}

export type GameOptionsSourceCandidate = {
	source_id: string
	instance_id: string
	name: string
	icon_path?: string | null
	game_version?: string | null
	eligible: boolean
	disabled_reason?: GameOptionsSourceDisabledReason | null
	recognized_setting_count: number
	custom_setting_count: number
}

export type GameOptionsSourceDisabledReason =
	| 'installing_or_updating'
	| 'running'
	| 'unsupported_version'
	| 'missing_options_file'
	| 'no_syncable_settings'
	| 'unreadable_options_file'

export type SaveGameSettingsResult = {
	state?: GameSettingsEditorState
	applied: number
	migrated: number
	deferred: number
	unsupported: number
	failed: number
	conflicts?: string[]
}

export async function list_game_options_sync_sources(): Promise<GameOptionsSourceCandidate[]> {
	return await invoke('plugin:instance|instance_list_game_options_sync_sources')
}

export async function get_synced_game_options_config(): Promise<GameSettingsEditorState> {
	return await invoke('plugin:instance|instance_get_synced_game_options_config')
}

export async function preview_synced_game_option_changes(
	request: UpdateGameSettingsRequest,
): Promise<GameSettingsEditorState> {
	return await invoke('plugin:instance|instance_preview_synced_game_option_changes', { request })
}

export async function save_synced_game_option_changes(
	request: UpdateGameSettingsRequest,
): Promise<SaveGameSettingsResult> {
	return await invoke('plugin:instance|instance_save_synced_game_option_changes', { request })
}
