import type { UserPreferencesContext } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'

type UserPreferences = NonNullable<UserPreferencesContext['preferences']['value']>
type PartialUserPreferences = Parameters<UserPreferencesContext['updatePreferences']>[0]

export async function get_user_preferences(userId: string): Promise<UserPreferences> {
	return await invoke<UserPreferences>('plugin:users|get_user_preferences', { userId })
}

export async function patch_user_preferences(
	userId: string,
	preferences: PartialUserPreferences,
): Promise<UserPreferences> {
	return await invoke<UserPreferences>('plugin:users|patch_user_preferences', {
		userId,
		preferences,
	})
}
