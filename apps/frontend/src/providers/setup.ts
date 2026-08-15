import { provideNotificationManager } from '@modrinth/ui'

import { FrontendNotificationManager } from './frontend-notifications'
import { setupAuthProvider } from './setup/auth'
import { setupFilePickerProvider } from './setup/file-picker'
import { setupLoadingStateProvider } from './setup/loading-state'
import { setupModrinthClientProvider } from './setup/modrinth-client'
import { setupPageContextProvider } from './setup/page-context'
import { setupTagsProvider } from './setup/tags'
import { setupUserCountryProvider } from './setup/user-country'

export function setupProviders(auth: Awaited<ReturnType<typeof useAuth>>) {
	provideNotificationManager(new FrontendNotificationManager())

	setupAuthProvider(auth)
	setupModrinthClientProvider(auth)
	setupTagsProvider()
	setupFilePickerProvider()
	setupPageContextProvider()
	setupLoadingStateProvider()
	setupUserCountryProvider()
}
