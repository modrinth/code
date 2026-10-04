import {
	BoxIcon,
	CurseForgeIcon,
	FolderOpenIcon,
	LinkIcon,
	RefreshCwIcon,
	UploadIcon,
} from '@modrinth/assets'
import { computed, type ComputedRef } from 'vue'

import type { ButtonMenuOption } from '#ui/components/index.ts'
import { defineMessages, useVIntl } from '#ui/composables/i18n.ts'
import { commonMessages } from '#ui/utils/common-messages.ts'

export function useFileActions(
	createFile: (type: 'file' | 'directory') => void,
	upload?: (type: 'file' | 'zip') => void,
	unzip?: (type: 'cf' | 'zip') => void,
	refresh?: { handleRefresh: () => void; isRefreshing: ComputedRef<boolean> },
): { options: ComputedRef<ButtonMenuOption[]> } {
	const { formatMessage } = useVIntl()

	const messages = defineMessages({
		createNew: {
			id: 'files.navbar.create-new',
			defaultMessage: 'Create new...',
		},
		newFile: {
			id: 'files.navbar.new-file',
			defaultMessage: 'New file',
		},
		newFolder: {
			id: 'files.navbar.new-folder',
			defaultMessage: 'New folder',
		},
		uploadFile: {
			id: 'files.navbar.upload-file',
			defaultMessage: 'Upload file',
		},
		uploadFromZip: {
			id: 'files.navbar.upload-from-zip',
			defaultMessage: 'Upload from .zip file',
		},
		uploadFromZipUrl: {
			id: 'files.navbar.upload-from-zip-url',
			defaultMessage: 'Upload from .zip URL',
		},
		installCurseForgePack: {
			id: 'files.navbar.install-curseforge-pack',
			defaultMessage: 'Install CurseForge pack',
		},
	})

	const canHandleZips = unzip != null
	const canHandleUpload = upload != null
	const canRefresh = refresh != null

	return {
		options: computed(() => {
			return [
				{
					id: 'file',
					icon: BoxIcon,
					label: formatMessage(messages.newFile),
					action: () => createFile('file'),
				},
				{
					id: 'directory',
					icon: FolderOpenIcon,
					label: formatMessage(messages.newFolder),
					action: () => createFile('directory'),
				},
				{
					id: 'upload',
					icon: UploadIcon,
					label: formatMessage(messages.uploadFile),
					shown: canHandleUpload,
					action: () => upload?.('file'),
				},
				{ type: 'divider', shown: canHandleZips },
				{
					id: 'install-from-url',
					icon: LinkIcon,
					label: formatMessage(messages.uploadFromZipUrl),
					shown: canHandleZips,
					action: () => unzip?.('zip'),
				},
				{
					id: 'install-cf-pack',
					icon: CurseForgeIcon,
					label: formatMessage(messages.installCurseForgePack),
					shown: canHandleZips,
					action: () => unzip?.('cf'),
				},
				{ type: 'divider', shown: canRefresh },
				{
					id: 'refresh',
					icon: RefreshCwIcon,
					iconClass: computed(() => (refresh?.isRefreshing.value ? 'animate-spin' : '')),
					label: formatMessage(commonMessages.refreshButton),
					shown: canRefresh,
					action: () => refresh?.handleRefresh(),
					disabled: refresh?.isRefreshing.value,
				},
			]
		}),
	}
}
