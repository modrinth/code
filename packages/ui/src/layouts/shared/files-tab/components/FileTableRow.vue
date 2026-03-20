<template>
	<li
		role="button"
		:class="[
			containerClasses,
			isDragSource ? 'opacity-50' : '',
		]"
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
			<ButtonStyled circular type="transparent">
				<TeleportOverflowMenu :options="menuOptions">
					<MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
					<template #extract
						><PackageOpenIcon /> {{ formatMessage(messages.extractLabel) }}</template
					>
					<template #rename><EditIcon /> {{ formatMessage(messages.renameLabel) }}</template>
					<template #move><RightArrowIcon /> {{ formatMessage(messages.moveLabel) }}</template>
					<template #download
						><DownloadIcon />
						{{ ctx.downloadButtonLabel ?? formatMessage(commonMessages.downloadButton) }}</template
					>
					<template #delete><TrashIcon /> {{ formatMessage(commonMessages.deleteLabel) }}</template>
				</TeleportOverflowMenu>
			</ButtonStyled>
		</div>
	</li>
</template>

<script setup lang="ts">
import {
	BoxIcon,
	BracesIcon,
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
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { getFileExtensionIcon } from '#ui/utils/auto-icons'
import { commonMessages } from '#ui/utils/common-messages'
import {
	getFileExtension,
	isEditableFile as isEditableFileExt,
	isImageFile,
} from '#ui/utils/file-extensions'

import { fileDragActive, fileDragData, fileDragTarget, startFileDrag, wasRecentDrag } from '../composables/file-drag-state'
import { injectFileManager } from '../providers/file-manager'

interface FileItemProps {
	name: string
	type: 'directory' | 'file'
	size?: number
	count?: number
	modified: number
	created: number
	path: string
	index: number
	isLast: boolean
	selected: boolean
	writeDisabled?: boolean
	writeDisabledTooltip?: string
}

const { formatMessage } = useVIntl()
const ctx = injectFileManager()

const messages = defineMessages({
	extractLabel: {
		id: 'files.row.extract',
		defaultMessage: 'Extract',
	},
	renameLabel: {
		id: 'files.row.rename',
		defaultMessage: 'Rename',
	},
	moveLabel: {
		id: 'files.row.move',
		defaultMessage: 'Move',
	},
	itemCount: {
		id: 'files.row.item-count',
		defaultMessage: '{count, plural, one {# item} other {# items}}',
	},
})

const props = defineProps<FileItemProps>()

const emit = defineEmits<{
	rename: [item: { name: string; type: string; path: string }]
	move: [item: { name: string; type: string; path: string }]
	download: [item: { name: string; type: string; path: string }]
	delete: [item: { name: string; type: string; path: string }]
	edit: [item: { name: string; type: string; path: string }]
	extract: [item: { name: string; type: string; path: string }]
	hover: [item: { name: string; type: string; path: string }]
	navigate: [item: { name: string; type: string; path: string }]
	moveDirectTo: [item: { name: string; type: string; path: string; destination: string }]
	contextmenu: [x: number, y: number]
	'toggle-select': []
}>()

const isDropTarget = computed(() => fileDragActive.value && fileDragTarget.value === props.path && props.type === 'directory')
const isDragSource = computed(() => fileDragActive.value && fileDragData.value?.path === props.path)

const units = Object.freeze(['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB'])

const formatDateTime = useFormatDateTime({
	year: '2-digit',
	month: '2-digit',
	day: '2-digit',
	hour: 'numeric',
	minute: 'numeric',
})

const containerClasses = computed(() => {
	const dropTarget = isDropTarget.value
	return [
		'group m-0 flex w-full select-none items-center justify-between overflow-hidden border-0 border-t border-solid border-surface-4 px-3 py-3 focus:!outline-none',
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

const menuOptions = computed(() => {
	const item = { name: props.name, type: props.type, path: props.path }
	const wd = props.writeDisabled
	const wdTooltip = props.writeDisabledTooltip
	return [
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
	const bytes = props.size
	if (bytes === 0) return '0 B'

	const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
	const size = (bytes / Math.pow(1024, exponent)).toFixed(2)
	return `${size} ${units[exponent]}`
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
				type: source.type,
				path: source.path,
				destination,
			})
		},
	)
}
</script>
