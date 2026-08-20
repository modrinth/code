import {
	type AbstractWebNotificationManager,
	type AuthProvider,
	setupUserPreferencesProvider,
} from '@modrinth/ui'

import { get_user_preferences, patch_user_preferences } from '@/helpers/user-preferences.ts'

export function setupAppUserPreferencesProvider(
	auth: AuthProvider,
	notificationManager: AbstractWebNotificationManager,
) {
	return setupUserPreferencesProvider({
		auth,
		getPreferences: get_user_preferences,
		patchPreferences: patch_user_preferences,
		notificationManager,
	})
}
