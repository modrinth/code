import type { AbstractModrinthClient } from '@modrinth/api-client'
import type { AbstractPopupNotificationManager, AbstractWebNotificationManager } from '@modrinth/ui'

import type { InstanceIconConfig } from '@/helpers/types'

import type { AppEvents } from './app-events'
import { setupOnboardingChecklistProvider } from './onboarding-checklist'
import { setupCreationModal } from './setup/creation-modal'
import { setupFileDownloadProvider } from './setup/file-download'
import { setupFileDropProvider } from './setup/file-drop'
import { setupFilePickerProvider } from './setup/file-picker'
import { setupIconCacheProvider } from './setup/icon-cache'
import { setupImageViewerEditorProvider } from './setup/image-viewer-editor'
import { setupInstanceImportProvider } from './setup/instance-import'
import { setupTagsProvider } from './setup/tags'
import { setupUserCountryProvider } from './setup/user-country'

export function setupProviders(
	client: AbstractModrinthClient,
	notificationManager: AbstractWebNotificationManager,
	_popupNotificationManager: AbstractPopupNotificationManager,
	appEvents: AppEvents,
	getGeneratedIconConfig?: (iconPath: string) => InstanceIconConfig | null,
) {
	setupUserCountryProvider(client)
	const tags = setupTagsProvider(notificationManager)
	setupFileDropProvider()
	setupFileDownloadProvider(client)
	setupFilePickerProvider()
	const iconCache = setupIconCacheProvider()
	setupImageViewerEditorProvider()
	setupInstanceImportProvider(notificationManager)
	const onboardingChecklist = setupOnboardingChecklistProvider(appEvents)

	return {
		...setupCreationModal(notificationManager, getGeneratedIconConfig),
		iconCache,
		onboardingChecklist,
		tags,
	}
}
