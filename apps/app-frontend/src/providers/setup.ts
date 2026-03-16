import type { AbstractWebNotificationManager } from '@modrinth/ui'

import { setupCreationModal } from './setup/creation-modal'
import { setupFilePickerProvider } from './setup/file-picker'
import { setupInstanceImportProvider } from './setup/instance-import'
import { setupTagsProvider } from './setup/tags'

export function setupProviders(notificationManager: AbstractWebNotificationManager) {
	setupTagsProvider(notificationManager)
	setupFilePickerProvider()
	setupInstanceImportProvider(notificationManager)

	return {
		...setupCreationModal(notificationManager),
	}
}
