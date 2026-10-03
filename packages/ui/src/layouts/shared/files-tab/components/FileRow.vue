<template>
	<Tooltip :action-wait="{hover: 600, unhover: 200}">
		<li
			role="option"
			:class="[containerClasses, isDragSource ? 'opacity-50' : '', compact ? 'h-8' : 'h-[3.25rem]', ]"
			:style="depth != null ? { paddingLeft: `${(depth * basePaddingFactor)}rem`} : undefined"
			tabindex="0"
			:data-file-path="path"
			:data-file-type="type"
			:aria-expanded="canExpand ? expanded : undefined"
			:aria-current="active ? 'location' : undefined"
			@click="selectItem"
			@auxclick="handleAuxClick"
			@contextmenu="openContextMenu"
			@keydown="handleKeydown"
			@mouseenter="handleMouseEnter"
			@mouseleave="handleMouseLeave"
			@pointerdown="handlePointerDown"
		>
			<span
				v-for="level in depth ?? 0"
				:key="level"
				aria-hidden="true"
				class="pointer-events-none absolute inset-y-0 w-px"
				:class="level === activeGuideLevel ? 'bg-brand-highlight' : 'bg-surface-5'"
				:style="{
					left: `${((level * basePaddingFactor) - 0.125)}rem`,
					height: `calc(2rem + ${isLast && level == depth ? '0rem' : '(0.25rem / 2)'})`,
					top: index != 0 ? `calc(-0.25rem / 2)` : ''
				}"
			/>
			<div class="pointer-events-none flex flex-1 items-center truncate h-full" :class="compact ? 'gap-2' : 'gap-3'">
				<button
					v-if="canExpand"
					type="button"
					tabindex="-1"
					class="pointer-events-auto -mr-1 flex size-5 shrink-0 items-center justify-center rounded border-none bg-transparent p-0 text-secondary hover:bg-surface-5 hover:text-contrast h-full"
					:aria-label="formatMessage(expanded ? messages.collapseFolder : messages.expandFolder)"
					@click.stop="emit('toggle-expand')"
					@pointerdown.stop
				>
					<ChevronRightIcon class="size-4 transition-transform duration-100" :class="{ 'rotate-90': expanded }" />
				</button>
				<span v-else-if="isTreeRow" class="-mr-1 size-5 shrink-0" aria-hidden="true" />
				<Checkbox
v-if="!selectionWithinActionMenu"
					class="pointer-events-auto"
					:model-value="selected"
					@click.stop
					@update:model-value="emit('toggle-select')"
				/>
				<div class="pointer-events-none flex size-5 shrink-0 items-center justify-center">
					<component
						:is="iconComponent"
						class="group-hover:text-contrast group-focus:text-contrast"
						:class="compact ? 'size-4' : 'size-5'"
					/>
				</div>
				<div class="pointer-events-none flex flex-col truncate">
					<span
						class="pointer-events-none truncate group-hover:text-contrast group-focus:text-contrast"
						:class="{ 'text-sm': compact, 'font-semibold text-contrast': active }"
					>
						{{ name }}
					</span>
				</div>
				<EditIcon v-if="hoveringToEdit && isEditableFile && !compact" />
			</div>
			<div v-if="!isTreeRow" class="pointer-events-auto flex w-fit flex-shrink-0 items-center gap-6">
				<span
					v-for="column in shownColumnDefinitions"
					:key="column.id"
					class="truncate text-nowrap text-sm text-secondary"
					:style="{ width: `${column.width}px` }"
				>
					{{ columnValues[column.id] }}
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
					<template v-if="type === 'directory'">
						<span class="text-nowrap text-sm text-secondary">
							{{ formatMessage(messages.items) }}
						</span>
						<span class="text-nowrap text-sm text-secondary">
							{{ columnValues.items }}
						</span>
					</template>
					<template v-else>
						<span class="text-nowrap text-sm text-secondary">
							{{ formatMessage(messages.size) }}
						</span>
						<span class="text-nowrap text-sm text-secondary">
							{{ columnValues.size }}
						</span>
					</template>
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
	ChevronRightIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
	FileIcon,
	FolderArchiveIcon,
	FolderCogIcon,
	FolderOpenIcon,
	GlassesIcon,
	GlobeIcon,
	MoreHorizontalIcon,
	PackageOpenIcon,
	PaintbrushIcon,
	PlusIcon,
	RightArrowIcon,
	TrashIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import { Tooltip } from "#ui/components";
import type {ButtonMenuLeafOption, ButtonMenuOption} from '#ui/components/base/buttons'
import { TeleportOverflowMenu } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import { useFormatBytes } from '#ui/composables'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectFileManager } from '#ui/layouts'
import { useFileActions } from "#ui/layouts/shared/files-tab/composables/folder-actions.ts";
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { getFileExtensionIcon } from '#ui/utils/auto-icons'
import { commonMessages } from '#ui/utils/common-messages'
import { canOpenInFileEditor, getFileExtension } from '#ui/utils/file-extensions'

import { FILE_COLUMNS, type FileColumn } from '../composables/file-columns'
import {
	fileDragActive,
	fileDragData,
	fileDragTarget,
	startFileDrag,
	wasRecentDrag,
} from '../composables/file-drag-state'
import type { FileItem } from '../types'
import { joinDisplayPath } from '../utils'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const ctx = injectFileManager()

const basePaddingFactor = 0.75;

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
	items: {
		id: 'files.table-header.items',
		defaultMessage: 'Items',
	},
	created: {
		id: 'files.table-header.created',
		defaultMessage: 'Created',
	},
	modified: {
		id: 'files.table-header.modified',
		defaultMessage: 'Modified',
	},
	openInNewTab: {
		id: 'files.row.open-in-new-tab',
		defaultMessage: 'Open in new tab',
	},
	newOrUpload: {
		id: 'files.row.upload-or-create',
		defaultMessage: 'New Entry',
	},
	expandFolder: {
		id: 'files.row.expand-folder',
		defaultMessage: 'Expand folder',
	},
	collapseFolder: {
		id: 'files.row.collapse-folder',
		defaultMessage: 'Collapse folder',
	},
})

const props = defineProps<
	FileItem & {
		index: number
		isLast: boolean
		selected: boolean
		writeDisabled?: boolean
		writeDisabledTooltip?: string
		/** Detail columns to render, in order. */
		columns?: FileColumn[]
		/** Whether some details aren't shown as columns, so hovering offers them in a tooltip. */
		hasHiddenDetails?: boolean
		selectionWithinActionMenu?: boolean
		/** Smaller row, used by the sidebar tree. */
		compact?: boolean
		/** Nesting level in the sidebar tree; indents the row. */
		depth?: number
		/** Renders as a sidebar tree row: expand chevron, no columns or inline actions menu. */
		isTreeRow?: boolean
		/** Whether this tree row's directory is expanded. */
		expanded?: boolean
		/** Whether this tree row's directory can be expanded, i.e. isn't known to be empty. */
		expandable?: boolean
		/** Highlights the row as the active tab's current location. */
		active?: boolean
		/** Tree guide level (1-based) to highlight, marking the directory the active location is in. */
		activeGuideLevel?: number
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
			| 'navigate'
			| 'open-in-new-tab',
		item: Pick<FileItem, 'name' | 'type' | 'path'>,
	): void
	(e: 'create', type: 'file' | 'directory'): void
	(e: 'upload', type: 'file' | 'zip'): void,
	(
		e: 'moveDirectTo',
		item: Pick<FileItem, 'name' | 'type' | 'path'>, destination: string,
	): void
	(e: 'contextmenu', event: MouseEvent, options: ButtonMenuOption[]): void
	(e: 'toggle-select'): void
	(e: 'toggle-expand'): void
}>()

const canExpand = computed(
	() => !!props.isTreeRow && props.type === 'directory' && props.expandable !== false,
)

const shownColumnDefinitions = computed(() =>
	(props.columns ?? []).flatMap((id) => FILE_COLUMNS.filter((column) => column.id === id)),
)
const canOpenInTab = computed(() => props.type === 'directory' || isEditableFile.value)

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
		'group relative m-0 flex w-full select-none items-center justify-between overflow-x-clip border-0 border-solid border-surface-4 pl-3 focus:!outline-none',
		props.compact ? 'rounded-lg pr-2 overflow-y-visible' : 'border-t py-2 pr-3 overflow-y-hidden',
		dropTarget
			? '!bg-brand-highlight'
			: props.active
				? 'bg-brand-highlight'
				: props.selected
					? 'bg-surface-2.5'
					: props.compact
						? 'bg-transparent'
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

const { options } = useFileActions(
	(type) => emit('create', type),
	(type) => emit('upload', type),
);

const menuOptions = computed<ButtonMenuOption[]>(() => {
	const item = { name: props.name, type: props.type, path: props.path }
	const wd = props.writeDisabled
	const wdTooltip = props.writeDisabledTooltip
	return [
		{
			type: "submenu",
			id: 'upload-create',
			label: formatMessage(messages.newOrUpload),
			icon: FileIcon,
			options: options.value as ButtonMenuLeafOption[],
		},
		{
			id: 'select-entry',
			label: formatMessage(commonMessages.selectEntryLabel),
			icon: iconComponent.value,
			action: () => {
				emit('toggle-select')
			},
			shown: props.selectionWithinActionMenu,
		},
		{
			id: 'open-in-new-tab',
			label: formatMessage(messages.openInNewTab),
			icon: PlusIcon,
			shown: canOpenInTab.value,
			action: () => emit('open-in-new-tab', item),
		},
		{ type: 'divider' },
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
			id: 'open-in-folder',
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

const columnValues = computed<Record<FileColumn, string>>(() => ({
	size: props.size != null ? formatBytes(props.size) : '',
	items:
		props.type === 'directory' ? formatMessage(messages.itemCount, { count: props.count ?? 0 }) : '',
	created: formattedCreationDate.value,
	modified: formattedModifiedDate.value,
}))

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

function selectItem(event?: MouseEvent) {
	if (wasRecentDrag()) return
	if (event?.ctrlKey || event?.metaKey) {
		emit('toggle-select')
		return
	}
	if (isNavigating.value) return
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

function handleAuxClick(event: MouseEvent) {
	if (event.button !== 1 || !canOpenInTab.value) return
	event.preventDefault()
	emit('open-in-new-tab', { name: props.name, type: props.type, path: props.path })
}

function handleKeydown(event: KeyboardEvent) {
	if (event.key === 'Enter') {
		selectItem()
	} else if (canExpand.value) {
		if (event.key === 'ArrowRight' && !props.expanded) emit('toggle-expand')
		if (event.key === 'ArrowLeft' && props.expanded) emit('toggle-expand')
	}
}

function handlePointerDown(e: PointerEvent) {
	if (e.button !== 0) return
	startFileDrag(
		{ name: props.name, type: props.type, path: props.path },
		e,
		(source, destination) => {
			emit('moveDirectTo', source, destination)
		},
	)
}
</script>
