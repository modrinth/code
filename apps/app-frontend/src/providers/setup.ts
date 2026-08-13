import type { AbstractModrinthClient } from '@modrinth/api-client'
import type { AbstractPopupNotificationManager, AbstractWebNotificationManager } from '@modrinth/ui'

import { setupCreationModal } from './setup/creation-modal'
import { setupFileDropProvider } from './setup/file-drop'
import { setupFilePickerProvider } from './setup/file-picker'
import { setupInstanceImportProvider } from './setup/instance-import'
import { setupTagsProvider } from './setup/tags'
import { setupUserCountryProvider } from './setup/user-country'

export function setupProviders(
	client: AbstractModrinthClient,
	notificationManager: AbstractWebNotificationManager,
	_popupNotificationManager: AbstractPopupNotificationManager,
) {
	setupUserCountryProvider(client)
	setupTagsProvider(notificationManager)
	setupFileDropProvider()
	setupFilePickerProvider()
	setupInstanceImportProvider(notificationManager)

	return {
		...setupCreationModal(notificationManager),
	}
}
