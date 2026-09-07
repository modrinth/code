import { provideNotificationManager, setupUserPreferencesProvider } from '@modrinth/ui'

import { FrontendNotificationManager } from './frontend-notifications'
import { setupAuthProvider } from './setup/auth'
import { setupFilePickerProvider } from './setup/file-picker'
import { setupLoadingStateProvider } from './setup/loading-state'
import { setupModrinthClientProvider } from './setup/modrinth-client'
import { setupPageContextProvider } from './setup/page-context'
import { setupTagsProvider } from './setup/tags'
import { setupUserCountryProvider } from './setup/user-country'

export function setupProviders(auth: Awaited<ReturnType<typeof useAuth>>) {
	const notificationManager = new FrontendNotificationManager()
	provideNotificationManager(notificationManager)

	const authProvider = setupAuthProvider(auth)
	const client = setupModrinthClientProvider(auth)
	const userPreferences = setupUserPreferencesProvider({
		auth: authProvider,
		getPreferences: (userId) => client.labrinth.users_v3.getPreferences(userId),
		patchPreferences: (userId, preferences) =>
			client.labrinth.users_v3.patchPreferences(userId, preferences),
		notificationManager,
	})
	setupTagsProvider()
	setupFilePickerProvider()
	setupPageContextProvider()
	setupLoadingStateProvider()
	setupUserCountryProvider()

	return { userPreferences, notificationManager }
}
