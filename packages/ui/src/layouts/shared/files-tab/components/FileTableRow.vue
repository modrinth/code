<template>
	<li
		role="button"
		:class="[containerClasses, isDragSource ? 'opacity-50' : '']"
		tabindex="0"
		:data-file-path="path"
		:data-file-type="type"
		@click="selectItem"
		@contextmenu="openContextMenu"
		@keydown="(e) => e.key === 'Enter' && selectItem()"
		@mouseenter="handleMouseEnter"
		@pointerdown="handlePointerDown"
	>
		<div class="pointer-events-none flex flex-1 items-center gap-3 truncate">
			<Checkbox
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
				<span
					class="pointer-events-none truncate group-hover:text-contrast group-focus:text-contrast"
				>
					{{ name }}
				</span>
			</div>
		</div>
		<div class="pointer-events-auto flex w-fit flex-shrink-0 items-center gap-4 @[800px]:gap-12">
			<span class="hidden w-[100px] text-nowrap text-sm text-secondary @[800px]:block">
				{{ formattedSize }}
			</span>
			<span class="hidden w-[160px] text-nowrap text-sm text-secondary @[800px]:block">
				{{ formattedCreationDate }}
			</span>
			<span class="hidden w-[160px] text-nowrap text-sm text-secondary @[800px]:block">
				{{ formattedModifiedDate }}
			</span>
			<div class="flex min-w-[51px] shrink-0 items-center justify-end">
				<ButtonStyled circular type="transparent">
					<TeleportOverflowMenu :options="menuOptions">
						<MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
						<template #copy-filename
							><ClipboardCopyIcon />
							{{ formatMessage(commonMessages.copyFilenameButton) }}</template
						>
						<template #copy-full-path
							><ClipboardCopyIcon />
							{{ formatMessage(commonMessages.copyFullPathButton) }}</template
						>
						<template #open-in-folder
							><FolderOpenIcon /> {{ formatMessage(commonMessages.openInFolderButton) }}</template
						>
						<template #extract
							><PackageOpenIcon /> {{ formatMessage(commonMessages.extractButton) }}</template
						>
						<template #rename
							><EditIcon /> {{ formatMessage(commonMessages.renameButton) }}</template
						>
						<template #move
							><RightArrowIcon /> {{ formatMessage(commonMessages.moveButton) }}</template
						>
						<template #download
							><DownloadIcon />
							{{
								ctx.downloadButtonLabel ?? formatMessage(commonMessages.downloadButton)
							}}</template
						>
						<template #delete
							><TrashIcon /> {{ formatMessage(commonMessages.deleteLabel) }}</template
						>
					</TeleportOverflowMenu>
				</ButtonStyled>
			</div>
		</div>
	</li>
</template>

<script setup lang="ts">
import {
	BoxIcon,
	BracesIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
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

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Checkbox from '#ui/components/base/Checkbox.vue'
import TeleportOverflowMenu from '#ui/components/base/TeleportOverflowMenu.vue'
import { useFormatBytes } from '#ui/composables'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { getFileExtensionIcon } from '#ui/utils/auto-icons'
import { commonMessages } from '#ui/utils/common-messages'
import {
	getFileExtension,
	isEditableFile as isEditableFileExt,
	isImageFile,
} from '#ui/utils/file-extensions'

import {
	fileDragActive,
	fileDragData,
	fileDragTarget,
	startFileDrag,
	wasRecentDrag,
} from '../composables/file-drag-state'
import { injectFileManager } from '../providers/file-manager'
import type { FileItem } from '../types'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const ctx = injectFileManager()

const messages = defineMessages({
	itemCount: {
		id: 'files.row.item-count',
		defaultMessage: '{count, plural, one {# item} other {# items}}',
	},
})

const props = defineProps<
	FileItem & {
		index: number
		isLast: boolean
		selected: boolean
		writeDisabled?: boolean
		writeDisabledTooltip?: string
	}
>()

const emit = defineEmits<{
	(
		e: 'rename' | 'move' | 'download' | 'delete' | 'edit' | 'extract' | 'hover' | 'navigate',
		item: Pick<FileItem, 'name' | 'type' | 'path'>,
	): void
	(
		e: 'moveDirectTo',
		item: Pick<FileItem, 'name' | 'type' | 'path'> & { destination: string },
	): void
	(e: 'contextmenu', x: number, y: number): void
	(e: 'toggle-select'): void
}>()

const isDropTarget = computed(
	() => fileDragActive.value && fileDragTarget.value === props.path && props.type === 'directory',
)
const isDragSource = computed(() => fileDragActive.value && fileDragData.value?.path === props.path)

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
		'group m-0 flex w-full select-none items-center justify-between overflow-hidden border-0 border-t border-solid border-surface-4 pl-3 pr-4 py-3 focus:!outline-none',
		dropTarget
			? '!bg-brand-highlight'
			: props.selected
				? 'bg-surface-2.5'
				: props.index % 2 === 0
					? 'bg-surface-2'
					: 'bg-surface-1.5',
		props.isLast ? 'rounded-b-[20px]' : '',
		isEditableFile.value || props.type === 'directory' ? 'cursor-pointer hover:bg-surface-2.5' : '',
		'transition-colors duration-100 focus:!outline-none',
	]
})

const fileExtension = computed(() => getFileExtension(props.name))

const isZip = computed(() => fileExtension.value === 'zip')

function getFullPath() {
	const basePath = ctx.basePath?.value
	return basePath ? `${basePath}/${props.path}`.replace(/\/+/g, '/') : props.path
}

const menuOptions = computed(() => {
	const item = { name: props.name, type: props.type, path: props.path }
	const wd = props.writeDisabled
	const wdTooltip = props.writeDisabledTooltip
	return [
		{
			id: 'copy-filename',
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
			icon: ClipboardCopyIcon,
			action: () => {
				navigator.clipboard.writeText(getFullPath())
				addNotification({ title: formatMessage(commonMessages.copiedPathLabel), type: 'success' })
			},
		},
		{
			id: 'open-in-folder',
			icon: FolderOpenIcon,
			shown: !!ctx.openInFolder,
			action: () => ctx.openInFolder?.(getFullPath()),
		},
		{ divider: true },
		{
			id: 'extract',
			shown: isZip.value,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => emit('extract', item),
		},
		{
			divider: true,
			shown: isZip.value,
		},
		{
			id: 'rename',
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => emit('rename', item),
		},
		{
			id: 'move',
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => emit('move', item),
		},
		{
			id: 'download',
			action: () => emit('download', item),
			shown: props.type !== 'directory',
		},
		{
			id: 'delete',
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => emit('delete', item),
			color: 'red' as const,
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
		const ext = fileExtension.value
		return !props.name.includes('.') || isEditableFileExt(ext) || isImageFile(ext)
	}
	return false
})

const formattedSize = computed(() => {
	if (props.type === 'directory') {
		return formatMessage(messages.itemCount, { count: props.count ?? 0 })
	}

	if (props.size === undefined) return ''
	return formatBytes(props.size)
})

function openContextMenu(event: MouseEvent) {
	event.preventDefault()
	emit('contextmenu', event.clientX, event.clientY)
}

function handleMouseEnter() {
	emit('hover', { name: props.name, type: props.type, path: props.path })
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
