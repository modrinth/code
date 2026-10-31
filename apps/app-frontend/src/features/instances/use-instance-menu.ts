import {
	ClipboardCopyIcon,
	EditIcon,
	EyeIcon,
	FolderOpenIcon,
	MinusIcon,
	PaletteIcon,
	PlayIcon,
	PlusIcon,
	StarIcon,
	StopCircleIcon,
	TrashIcon,
	UploadIcon,
} from '@modrinth/assets'
import {
	type ButtonMenuLeafOption,
	type ButtonMenuOption,
	defineMessages,
	injectFilePicker,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'

import { useHiddenInstances } from '@/features/library/hidden-instances/use-hidden-instances'
import { trackEvent } from '@/helpers/analytics'
import { toError } from '@/helpers/errors'
import { install_duplicate_instance } from '@/helpers/install'
import { edit, edit_icon } from '@/helpers/instance'
import { FAVORITES_GROUP_ID } from '@/helpers/instance-groups'

import { injectInstanceActions } from './instance-actions/context'
import type { InstanceCard } from './use-instance-actions'

const messages = defineMessages({
	play: { id: 'app.library.instance.action.play', defaultMessage: 'Play' },
	stop: { id: 'app.library.instance.action.stop', defaultMessage: 'Stop' },
	addToFavorites: {
		id: 'app.library.instance.action.add-to-favorites',
		defaultMessage: 'Add to favorites',
	},
	removeFromFavorites: {
		id: 'app.library.instance.action.remove-from-favorites',
		defaultMessage: 'Remove from favorites',
	},
	addContent: { id: 'app.library.instance.action.add-content', defaultMessage: 'Add content' },
	viewInstance: {
		id: 'app.library.instance.action.view-instance',
		defaultMessage: 'View instance',
	},
	editIcon: {
		id: 'instance.settings.tabs.general.edit-icon',
		defaultMessage: 'Edit icon',
	},
	selectIcon: {
		id: 'instance.settings.tabs.general.edit-icon.select',
		defaultMessage: 'Select icon',
	},
	replaceIcon: {
		id: 'instance.settings.tabs.general.edit-icon.replace',
		defaultMessage: 'Replace icon',
	},
	createIcon: {
		id: 'instance.settings.tabs.general.edit-icon.create',
		defaultMessage: 'Create an icon',
	},
	editCreatedIcon: {
		id: 'instance.settings.tabs.general.edit-icon.edit-created',
		defaultMessage: 'Edit icon',
	},
	removeIcon: {
		id: 'instance.settings.tabs.general.edit-icon.remove',
		defaultMessage: 'Remove icon',
	},
	duplicateInstance: {
		id: 'app.library.instance.action.duplicate',
		defaultMessage: 'Duplicate instance',
	},
	delete: { id: 'app.library.instance.action.delete', defaultMessage: 'Delete' },
	openFolder: { id: 'app.library.instance.action.open-folder', defaultMessage: 'Open folder' },
	copyPath: { id: 'app.library.instance.action.copy-path', defaultMessage: 'Copy path' },
	removeFromGroup: {
		id: 'app.library.instance.action.remove-from-group',
		defaultMessage: 'Remove from group',
	},
})

export type InstanceMenuOptions = {
	removeFromGroupId?: string | null
}

export function useInstanceMenu() {
	const { formatMessage } = useVIntl()
	const { handleError } = injectNotificationManager()
	const filePicker = injectFilePicker()
	const { confirmDelete, editIcon } = injectInstanceActions()
	const { getVisibilityOption } = useHiddenInstances()

	const setInstanceGroups = async (item: InstanceCard, groupIds: string[]) =>
		await edit(item.instance.id, { group_ids: groupIds }).catch((error) =>
			handleError(toError(error)),
		)

	const duplicateInstance = async (instanceId: string) => {
		await install_duplicate_instance(instanceId).catch((error) => handleError(toError(error)))
	}

	const selectInstanceIcon = async (item: InstanceCard) => {
		const iconPath = (await filePicker.pickImage())?.path
		if (!iconPath) return

		try {
			await edit_icon(item.instance.id, iconPath)
			trackEvent('InstanceSetIcon')
		} catch (error) {
			handleError(toError(error))
		}
	}

	const removeInstanceIcon = async (item: InstanceCard) => {
		try {
			await edit_icon(item.instance.id, null)
			trackEvent('InstanceRemoveIcon')
		} catch (error) {
			handleError(toError(error))
		}
	}

	const buildInstanceIconOptions = (item: InstanceCard): ButtonMenuLeafOption[] => [
		{
			id: item.instance.icon_path ? 'replace_icon' : 'select_icon',
			label: formatMessage(item.instance.icon_path ? messages.replaceIcon : messages.selectIcon),
			icon: UploadIcon,
			action: () => void selectInstanceIcon(item),
		},
		{
			id: item.instance.icon_config ? 'edit_created_icon' : 'create_icon',
			label: formatMessage(
				item.instance.icon_config ? messages.editCreatedIcon : messages.createIcon,
			),
			icon: PaletteIcon,
			action: () => editIcon(item.instance),
		},
		{
			id: 'remove_icon',
			label: formatMessage(messages.removeIcon),
			icon: TrashIcon,
			shown: !!item.instance.icon_path,
			action: () => void removeInstanceIcon(item),
		},
	]

	const getInstanceOptions = (
		item: InstanceCard,
		{ removeFromGroupId = null }: InstanceMenuOptions = {},
	): ButtonMenuOption[] => {
		const isFavorite = item.instance.group_ids.includes(FAVORITES_GROUP_ID)
		const installing = item.instance.install_stage.includes('installing')
		const canAddContent = !item.instance.quarantined && !item.instance.link && !installing

		return [
			{
				id: 'stop',
				label: formatMessage(messages.stop),
				icon: StopCircleIcon,
				shown: item.playing,
				tone: 'red',
				action: () => void item.stop(null, 'InstanceGridContextMenu'),
			},
			{
				id: 'play',
				label: formatMessage(messages.play),
				icon: PlayIcon,
				shown: !item.playing && !item.instance.quarantined,
				disabled: installing,
				tone: 'brand',
				action: () => void item.play(null, 'InstanceGridContextMenu'),
			},
			{
				id: isFavorite ? 'remove_from_favorites' : 'add_to_favorites',
				label: formatMessage(isFavorite ? messages.removeFromFavorites : messages.addToFavorites),
				icon: StarIcon,
				action: () =>
					void setInstanceGroups(
						item,
						isFavorite
							? item.instance.group_ids.filter((groupId) => groupId !== FAVORITES_GROUP_ID)
							: [...new Set([...item.instance.group_ids, FAVORITES_GROUP_ID])],
					),
			},
			{ type: 'divider' },
			{
				id: 'add_content',
				label: formatMessage(messages.addContent),
				icon: PlusIcon,
				shown: canAddContent,
				action: () => void item.addContent(),
			},
			{ type: 'divider', shown: canAddContent },
			{
				id: 'edit',
				label: formatMessage(messages.viewInstance),
				icon: EyeIcon,
				disabled: installing,
				action: () => void item.seeInstance(),
			},
			{
				id: 'duplicate',
				label: formatMessage(messages.duplicateInstance),
				icon: ClipboardCopyIcon,
				disabled: item.instance.install_stage !== 'installed',
				action: () => void duplicateInstance(item.instance.id),
			},
			{
				id: 'open',
				label: formatMessage(messages.openFolder),
				icon: FolderOpenIcon,
				action: () => void item.openFolder(),
			},
			{
				id: 'copy',
				label: formatMessage(messages.copyPath),
				icon: ClipboardCopyIcon,
				action: () => void navigator.clipboard.writeText(item.instance.id),
			},
			{
				type: 'submenu',
				id: 'edit_icon',
				label: formatMessage(messages.editIcon),
				icon: EditIcon,
				options: buildInstanceIconOptions(item),
			},
			{
				id: 'remove_from_group',
				label: formatMessage(messages.removeFromGroup),
				icon: MinusIcon,
				shown: !!removeFromGroupId,
				action: () =>
					void setInstanceGroups(
						item,
						item.instance.group_ids.filter((groupId) => groupId !== removeFromGroupId),
					),
			},
			getVisibilityOption(item.instance.id),
			{ type: 'divider' },
			{
				id: 'delete',
				label: formatMessage(messages.delete),
				icon: TrashIcon,
				tone: 'red',
				hoverFilledOnly: true,
				action: () => confirmDelete(item.instance),
			},
		]
	}

	return { getInstanceOptions }
}
