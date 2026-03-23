import { provideNotificationManager } from '@modrinth/ui'

import { FrontendNotificationManager } from './frontend-notifications'
import { setupFilePickerProvider } from './setup/file-picker'
import { setupModrinthClientProvider } from './setup/modrinth-client'
import { setupPageContextProvider } from './setup/page-context'
import { setupTagsProvider } from './setup/tags'

export function setupProviders(auth: Awaited<ReturnType<typeof useAuth>>) {
	provideNotificationManager(new FrontendNotificationManager())

	setupModrinthClientProvider(auth)
	setupTagsProvider()
	setupFilePickerProvider()
	setupPageContextProvider()
}
