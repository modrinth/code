<template>
	<Tooltip :disabled="containerWidth != null ? (containerWidth > 900 ? showDetails : false) : showDetails" :allow-hover="true">
		<li
			role="option"
			:class="[containerClasses, isDragSource ? 'opacity-50' : '']"
			tabindex="0"
			:data-file-path="path"
			:data-file-type="type"
			@click="selectItem"
			@contextmenu="openContextMenu"
			@keydown="(e) => e.key === 'Enter' && selectItem()"
			@mouseenter="handleMouseEnter"
			@mouseleave="handleMouseLeave"
			@pointerdown="handlePointerDown"
			class="h-[52.8px]"
		>
			<div class="pointer-events-none flex flex-1 items-center gap-3 truncate">
				<Checkbox v-if="!selectionWithinActionMenu"
					class="pointer-events-auto"
					:model-value="selected"
					@click.stop
					@update:model-value="emit('toggle-select')"
				/>
				<div class="pointer-events-none flex size-5 items-center justify-center">
					<component
						:is="iconComponent"
						class="size-5 group-hover:text-contrast group-focus:text-contrast"
					/>
				</div>
				<div class="pointer-events-none flex flex-col truncate">
					<span class="pointer-events-none truncate group-hover:text-contrast group-focus:text-contrast">
						{{ name }}
					</span>
				</div>
				<EditIcon v-if="hoveringToEdit && isEditableFile" />
			</div>
			<div class="pointer-events-auto flex w-fit flex-shrink-0 items-center gap-4 @[900px]:gap-12">
				<span v-if="showDetails" class="hidden w-[100px] text-nowrap text-sm text-secondary @[900px]:block">
					{{ formattedSize }}
				</span>
				<span v-if="showDetails" class="hidden w-[160px] text-nowrap text-sm text-secondary @[900px]:block">
					{{ formattedCreationDate }}
				</span>
				<span v-if="showDetails" class="hidden w-[160px] text-nowrap text-sm text-secondary @[900px]:block">
					{{ formattedModifiedDate }}
				</span>
				<div class="grid min-w-[51px] shrink-0 items-center justify-items-end">
					<span
						aria-hidden="true"
						class="invisible col-start-1 row-start-1 text-nowrap font-semibold"
					>
						{{ formatMessage(commonMessages.actionsLabel) }}
					</span>
					<div
						class="col-start-1 row-start-1 flex justify-end"
						@click.stop
						@contextmenu.stop
						@keydown.stop
						@pointerdown.stop
					>
						<TeleportOverflowMenu
							type="quiet"
							:label="formatMessage(commonMessages.actionsLabel)"
							:options="menuOptions"
						>
							<MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
						</TeleportOverflowMenu>
					</div>
				</div>
			</div>
		</li>
		<template #popper>
			<div class="flex flex-col w-fit">
				<h3 class="mb-2 pb-1 mt-0 text-base font-semibold border-0 border-b-[1px] border-solid border-divider">Details</h3>
				<div class="gap-1 grid grid-cols-2">
					<span class="text-nowrap text-sm text-secondary">
						{{ formatMessage(messages.size) }}
					</span>
					<span class="text-nowrap text-sm text-secondary">
						{{ formattedSize }}
					</span>
					<span class="text-nowrap text-sm text-secondary">
						{{ formatMessage(messages.created) }}
					</span>
					<span class="text-nowrap text-sm text-secondary">
						{{ formattedCreationDate }}
					</span>
					<span class="text-nowrap text-sm text-secondary">
						{{ formatMessage(messages.modified) }}
					</span>
					<span class="text-nowrap text-sm text-secondary">
						{{ formattedModifiedDate }}
					</span>
				</div>
			</div>
		</template>
	</Tooltip>
</template>

<script setup lang="ts">
import {
	BoxIcon,
	BracesIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
	FolderArchiveIcon,
	FolderCogIcon,
	FolderOpenIcon,
	GlassesIcon,
	GlobeIcon,
	MoreHorizontalIcon,
	PackageOpenIcon,
	PaintbrushIcon,
	RightArrowIcon,
	TrashIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import type { ButtonMenuOption } from '#ui/components/base/buttons'
import { TeleportOverflowMenu } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import { useFormatBytes } from '#ui/composables'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { getFileExtensionIcon } from '#ui/utils/auto-icons'
import { commonMessages } from '#ui/utils/common-messages'
import { canOpenInFileEditor, getFileExtension } from '#ui/utils/file-extensions'

import {
	fileDragActive,
	fileDragData,
	fileDragTarget,
	startFileDrag,
	wasRecentDrag,
} from '../composables/file-drag-state'
import { injectFileManager } from '../providers/file-manager'
import type { FileItem } from '../types'
import { joinDisplayPath } from '../utils'
import {Tooltip} from "#ui/components";

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const ctx = injectFileManager()

const messages = defineMessages({
	itemCount: {
		id: 'files.row.item-count',
		defaultMessage: '{count, plural, one {# item} other {# items}}',
	},
	createZip: {
		id: 'files.row.create-zip',
		defaultMessage: 'Create ZIP',
	},
	size: {
		id: 'files.table-header.size',
		defaultMessage: 'Size',
	},
	created: {
		id: 'files.table-header.created',
		defaultMessage: 'Created',
	},
	modified: {
		id: 'files.table-header.modified',
		defaultMessage: 'Modified',
	},
})

const props = defineProps<
	FileItem & {
		index: number
		isLast: boolean
		selected: boolean
		writeDisabled?: boolean
		writeDisabledTooltip?: string
		showDetails: boolean
		containerWidth?: number | undefined
		selectionWithinActionMenu?: boolean
	}
>()

const emit = defineEmits<{
	(
		e:
			| 'rename'
			| 'move'
			| 'download'
			| 'zip'
			| 'delete'
			| 'edit'
			| 'extract'
			| 'hover'
			| 'navigate',
		item: Pick<FileItem, 'name' | 'type' | 'path'>,
	): void
	(
		e: 'moveDirectTo',
		item: Pick<FileItem, 'name' | 'type' | 'path'> & { destination: string },
	): void
	(e: 'contextmenu', event: MouseEvent, options: ButtonMenuOption[]): void
	(e: 'toggle-select'): void
}>()

const isDropTarget = computed(
	() => fileDragActive.value && fileDragTarget.value === props.path && props.type === 'directory',
)
const isDragSource = computed(() => fileDragActive.value && fileDragData.value?.path === props.path)
const hoveringToEdit = ref<boolean>(false);

const formatDateTime = useFormatDateTime({
	year: '2-digit',
	month: '2-digit',
	day: '2-digit',
	hour: 'numeric',
	minute: 'numeric',
})
const formatBytes = useFormatBytes()

const containerClasses = computed(() => {
	const dropTarget = isDropTarget.value
	return [
		'group m-0 flex w-full select-none items-center justify-between overflow-hidden border-0 border-t border-solid border-surface-4 pl-3 pr-3 py-2 focus:!outline-none',
		dropTarget
			? '!bg-brand-highlight'
			: props.selected
				? 'bg-surface-2.5'
				: props.index % 2 === 0
					? 'bg-surface-2'
					: 'bg-surface-1.5',
		props.isLast ? '' : '',
		isEditableFile.value || props.type === 'directory' ? 'cursor-pointer hover:bg-surface-2.5' : '',
		'transition-colors duration-100 focus:!outline-none',
	]
})

const fileExtension = computed(() => getFileExtension(props.name))

const canExtract = computed(() => fileExtension.value === 'zip' && !!ctx.extractFile)

function getFullPath() {
	return joinDisplayPath(ctx.basePath?.value, props.path)
}

const menuOptions = computed<ButtonMenuOption[]>(() => {
	const item = { name: props.name, type: props.type, path: props.path }
	const wd = props.writeDisabled
	const wdTooltip = props.writeDisabledTooltip
	return [
		{
			id: 'copy-filename',
			label: formatMessage(commonMessages.selectEntryLabel),
			icon: iconComponent.value,
			action: () => {
				emit('toggle-select')
			},
			shown: props.selectionWithinActionMenu,
		},
		{
			id: 'copy-filename',
			label: formatMessage(commonMessages.copyFilenameButton),
			icon: ClipboardCopyIcon,
			action: () => {
				navigator.clipboard.writeText(props.name)
				addNotification({
					title: formatMessage(commonMessages.copiedFilenameLabel),
					type: 'success',
				})
			},
		},
		{
			id: 'copy-full-path',
			label: formatMessage(commonMessages.copyFullPathButton),
			icon: ClipboardCopyIcon,
			action: () => {
				navigator.clipboard.writeText(getFullPath())
				addNotification({ title: formatMessage(commonMessages.copiedPathLabel), type: 'success' })
			},
		},
		{
			id: 'select-entry',
			label: formatMessage(commonMessages.openInFolderButton),
			icon: FolderOpenIcon,
			shown: !!ctx.openInFolder,
			action: () => ctx.openInFolder?.(getFullPath()),
		},
		{ type: 'divider' },
		{
			id: 'extract',
			label: formatMessage(commonMessages.extractButton),
			icon: PackageOpenIcon,
			shown: canExtract.value,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => emit('extract', item),
		},
		{ type: 'divider', shown: canExtract.value },
		{
			id: 'zip',
			label: formatMessage(messages.createZip),
			icon: FolderArchiveIcon,
			shown: props.type === 'directory' && !!ctx.zipFolder,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => emit('zip', item),
		},
		{ type: 'divider', shown: props.type === 'directory' && !!ctx.zipFolder },
		{
			id: 'rename',
			label: formatMessage(commonMessages.renameButton),
			icon: EditIcon,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => emit('rename', item),
		},
		{
			id: 'move',
			label: formatMessage(commonMessages.moveButton),
			icon: RightArrowIcon,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => emit('move', item),
		},
		{
			id: 'download',
			label: ctx.downloadButtonLabel ?? formatMessage(commonMessages.downloadButton),
			icon: DownloadIcon,
			action: () => emit('download', item),
			shown: props.type !== 'directory',
		},
		{
			id: 'delete',
			label: formatMessage(commonMessages.deleteLabel),
			icon: TrashIcon,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => emit('delete', item),
			tone: 'red',
		},
	]
})

const iconComponent = computed(() => {
	if (props.type === 'directory') {
		if (props.name === 'config') return FolderCogIcon
		if (props.name === 'world' || props.name === 'saves') return GlobeIcon
		if (props.name === 'mods') return BoxIcon
		if (props.name === 'resourcepacks') return PaintbrushIcon
		if (props.name === 'shaderpacks') return GlassesIcon
		if (props.name === 'datapacks') return BracesIcon
		return FolderOpenIcon
	}

	return getFileExtensionIcon(fileExtension.value)
})

const formattedModifiedDate = computed(() => {
	const date = new Date(props.modified * 1000)
	return formatDateTime(date)
})

const formattedCreationDate = computed(() => {
	const date = new Date(props.created * 1000)
	return formatDateTime(date)
})

const isEditableFile = computed(() => {
	if (props.type === 'file') {
		return canOpenInFileEditor(props.name)
	}
	return false
})

const formattedSize = computed(() => {
	if (props.type === 'directory') {
		return formatMessage(messages.itemCount, { count: props.count ?? 0 })
	}

	if (props.size == null) return ''
	return formatBytes(props.size)
})

function openContextMenu(event: MouseEvent) {
	event.preventDefault()
	emit('contextmenu', event, menuOptions.value)
}

function handleMouseEnter() {
	hoveringToEdit.value = true;
	emit('hover', { name: props.name, type: props.type, path: props.path })
}

function handleMouseLeave() {
	hoveringToEdit.value = false;
}

const isNavigating = ref(false)

function selectItem() {
	if (isNavigating.value || wasRecentDrag()) return
	isNavigating.value = true

	const item = { name: props.name, type: props.type, path: props.path }
	if (props.type === 'directory') {
		emit('navigate', item)
	} else if (props.type === 'file' && isEditableFile.value) {
		emit('edit', item)
	}

	setTimeout(() => {
		isNavigating.value = false
	}, 500)
}

function handlePointerDown(e: PointerEvent) {
	if (e.button !== 0) return
	startFileDrag(
		{ name: props.name, type: props.type, path: props.path },
		e,
		(source, destination) => {
			emit('moveDirectTo', {
				name: source.name,
				type: source.type as FileItem['type'],
				path: source.path,
				destination,
			})
		},
	)
}
</script>
