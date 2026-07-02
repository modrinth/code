import type { AbstractPopupNotificationManager, AbstractWebNotificationManager } from '@modrinth/ui'

import { setupCreationModal } from './setup/creation-modal'
import { setupFileDropProvider } from './setup/file-drop'
import { setupFilePickerProvider } from './setup/file-picker'
import { setupInstanceImportProvider } from './setup/instance-import'
import { setupTagsProvider } from './setup/tags'

export function setupProviders(
	notificationManager: AbstractWebNotificationManager,
	_popupNotificationManager: AbstractPopupNotificationManager,
) {
	setupTagsProvider(notificationManager)
	setupFileDropProvider()
	setupFilePickerProvider()
	setupInstanceImportProvider(notificationManager)

	return {
		...setupCreationModal(notificationManager),
	}
}
