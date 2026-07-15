import type { AbstractPopupNotificationManager, AbstractWebNotificationManager } from '@modrinth/ui'

import { setupOnboardingChecklistProvider } from './onboarding-checklist'
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
	const onboardingChecklist = setupOnboardingChecklistProvider()

	return {
		...setupCreationModal(notificationManager),
		onboardingChecklist,
	}
}
