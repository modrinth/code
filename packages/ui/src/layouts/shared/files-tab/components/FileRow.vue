<template>
	<Tooltip
		:delay="{ hover: 850, unhover: 100 }"
		:hoverable="true"
		:allow-transfer="false"
		:disabled="hiddenDetails.length === 0"
	>
		<li
			role="option"
			:class="[containerClasses, isDragSource ? 'opacity-50' : '', compact ? 'h-8' : 'h-[3.25rem]']"
			:style="depth != null ? { paddingLeft: `${depth * basePaddingFactor}rem` } : undefined"
			tabindex="0"
			:data-file-path="file.path"
			:data-file-type="file.type"
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
					left: `${level * basePaddingFactor - 0.125}rem`,
					height: `calc(2rem + ${isLast && level == depth ? '0rem' : '(0.25rem / 2)'})`,
					top: index != 0 ? `calc(-0.25rem / 2)` : '',
				}"
			/>
			<div
				class="pointer-events-none flex flex-1 items-center truncate h-full"
				:class="compact ? 'gap-2' : 'gap-3'"
			>
				<button
					v-if="canExpand"
					type="button"
					tabindex="-1"
					class="pointer-events-auto -mr-1 flex size-5 shrink-0 items-center justify-center rounded border-none bg-transparent p-0 text-secondary hover:bg-surface-5 hover:text-contrast h-full"
					:aria-label="formatMessage(expanded ? messages.collapseFolder : messages.expandFolder)"
					@click.stop="emit('toggle-expand', file)"
					@pointerdown.stop
				>
					<ChevronRightIcon
						class="size-4 transition-transform duration-100"
						:class="{ 'rotate-90': expanded }"
					/>
				</button>
				<span v-else-if="isTreeRow" class="-mr-1 size-5 shrink-0" aria-hidden="true" />
				<Checkbox
					v-if="!selectionWithinActionMenu"
					class="pointer-events-auto"
					:model-value="selected"
					@click.stop
					@update:model-value="ui.toggleItemSelection(file)"
				/>
				<div class="pointer-events-none flex size-5 shrink-0 items-center justify-center">
					<component
						:is="iconStyle.icon"
						:class="[
							ui.coloredIcons.value
								? iconStyle.color
								: 'group-hover:text-contrast group-focus:text-contrast',
							compact ? 'size-4' : 'size-5',
						]"
					/>
				</div>
				<div class="pointer-events-none flex flex-col truncate">
					<span
						class="pointer-events-none truncate group-hover:text-contrast group-focus:text-contrast"
						:class="{ 'text-sm': compact, 'font-semibold text-contrast': active }"
					>
						{{ file.name }}
					</span>
				</div>
				<EditIcon v-if="hoveringToEdit && isEditableFile && !compact" />
			</div>
			<div
				v-if="!isTreeRow"
				class="pointer-events-auto flex w-fit flex-shrink-0 items-center gap-6"
			>
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
				<h3
					class="mb-2 pb-1 mt-0 text-base font-semibold border-0 border-b-[1px] border-solid border-divider"
				>
					{{ formatMessage(messages.details) }}
				</h3>
				<div v-for="key in hiddenDetails" :key="key" class="gap-1 grid grid-cols-2">
					<span class="text-nowrap text-sm text-secondary">
						{{ formatMessage(messages[key]) }}
					</span>
					<span class="text-nowrap text-sm text-secondary">
						{{ columnValues[key] }}
					</span>
				</div>
			</div>
		</template>
	</Tooltip>
</template>

<script setup lang="ts">
import {
	ChevronRightIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
	FileIcon,
	FolderArchiveIcon,
	FolderOpenIcon,
	MoreHorizontalIcon,
	PackageOpenIcon,
	PlusIcon,
	RightArrowIcon,
	TrashIcon,
} from '@modrinth/assets'
import { computed, ref, toValue } from 'vue'

import { Tooltip } from '#ui/components'
import type { ButtonMenuLeafOption, ButtonMenuOption } from '#ui/components/base/buttons'
import { TeleportOverflowMenu } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import { useFormatBytes } from '#ui/composables'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useFileActions } from '#ui/layouts/shared/files-tab/composables/folder-actions.ts'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { commonMessages } from '#ui/utils/common-messages'
import { canOpenInFileEditor, getFileExtension } from '#ui/utils/file-extensions'

import { FILE_COLUMNS, FILE_COLUMNS_ORDER, type FileColumn } from '../composables/file-columns'
import {
	fileDragActive,
	fileDragData,
	fileDragTarget,
	startFileDrag,
	wasRecentDrag,
} from '../composables/file-drag-state'
import { injectFileBrowserUI } from '../providers/file-browser-ui'
import type { FileItem } from '../types'
import { childPath, fileIconFor, infoFrom, joinDisplayPath } from '../utils'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const ui = injectFileBrowserUI()

const basePaddingFactor = 0.75

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
	details: {
		id: 'files.table-header.details',
		defaultMessage: 'Details',
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

const props = defineProps<{
	index: number
	file: FileItem
	isLast: boolean
	/** Detail columns to render, in order. Details not shown as columns are offered in a tooltip. */
	columns?: FileColumn[]
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
}>()

/**
 * Row actions go straight to the browser UI context. Only what differs between the listing and
 * the sidebar tree is emitted, with the row's file as payload, so parents can bind handlers by
 * reference; inline handlers would re-render every row whenever the parent renders.
 */
const emit = defineEmits<{
	navigate: [file: FileItem]
	hover: [file: FileItem]
	'toggle-expand': [file: FileItem]
}>()

const selected = computed(() => ui.selectedItems.value.has(props.file.path))
const readOnly = computed(() => !!ui.isReadOnly?.(props.file))
const writeDisabled = computed(() => ui.isBusy.value || readOnly.value)
const writeDisabledTooltip = computed(() =>
	readOnly.value ? toValue(ui.readOnlyReason) : ui.busyTooltip.value,
)

const canExpand = computed(
	() => !!props.isTreeRow && props.file.type === 'directory' && props.expandable !== false,
)

const shownColumnDefinitions = computed(() =>
	(props.columns ?? []).flatMap((id) => FILE_COLUMNS.filter((column) => column.id === id)),
)
const canOpenInTab = computed(
	() => ui.advancedView.value && (props.file.type === 'directory' || isEditableFile.value),
)

const isDropTarget = computed(
	() =>
		fileDragActive.value &&
		fileDragTarget.value === props.file.path &&
		props.file.type === 'directory',
)
const isDragSource = computed(
	() => fileDragActive.value && fileDragData.value?.path === props.file.path,
)
const hoveringToEdit = ref<boolean>(false)

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
				: selected.value
					? 'bg-surface-2.5'
					: props.compact
						? 'bg-transparent'
						: props.index % 2 === 0
							? 'bg-surface-2'
							: 'bg-surface-1.5',
		props.isLast ? '' : '',
		isEditableFile.value || props.file.type === 'directory'
			? 'cursor-pointer hover:bg-surface-2.5'
			: '',
		'transition-colors duration-100 focus:!outline-none',
	]
})

const fileExtension = computed(() => getFileExtension(props.file.name))

const canExtract = computed(() => fileExtension.value === 'zip' && !!ui.extractFile)

function getFullPath() {
	return joinDisplayPath(toValue(ui.basePath), props.file.path)
}

const { options } = useFileActions(
	(type) => ui.showCreateModal(type),
	() => ui.initiateFileUpload(),
)

const menuOptions = computed<ButtonMenuOption[]>(() => {
	const item = infoFrom(props.file)
	const wd = writeDisabled.value
	const wdTooltip = writeDisabledTooltip.value
	return [
		{
			type: 'submenu',
			id: 'upload-create',
			label: formatMessage(messages.newOrUpload),
			icon: FileIcon,
			options: options.value as ButtonMenuLeafOption[],
		},
		{
			id: 'select-entry',
			label: formatMessage(commonMessages.selectEntryLabel),
			icon: iconStyle.value.icon,
			action: () => ui.toggleItemSelection(props.file),
			shown: props.selectionWithinActionMenu,
		},
		{
			id: 'open-in-new-tab',
			label: formatMessage(messages.openInNewTab),
			icon: PlusIcon,
			shown: canOpenInTab.value,
			action: () => ui.handleOpenInNewTab(item),
		},
		{ type: 'divider' },
		{
			id: 'copy-filename',
			label: formatMessage(commonMessages.copyFilenameButton),
			icon: ClipboardCopyIcon,
			action: () => {
				navigator.clipboard.writeText(props.file.name)
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
			shown: !!ui.openInFolder,
			action: () => ui.openInFolder?.(getFullPath()),
		},
		{ type: 'divider' },
		{
			id: 'extract',
			label: formatMessage(commonMessages.extractButton),
			icon: PackageOpenIcon,
			shown: canExtract.value,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => ui.handleExtractItem(item),
		},
		{ type: 'divider', shown: canExtract.value },
		{
			id: 'zip',
			label: formatMessage(messages.createZip),
			icon: FolderArchiveIcon,
			shown: props.file.type === 'directory' && !!ui.zipFolder,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => ui.zipFolder?.(item),
		},
		{ type: 'divider', shown: props.file.type === 'directory' && !!ui.zipFolder },
		{
			id: 'rename',
			label: formatMessage(commonMessages.renameButton),
			icon: EditIcon,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => ui.showRenameModal(props.file),
		},
		{
			id: 'move',
			label: formatMessage(commonMessages.moveButton),
			icon: RightArrowIcon,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => ui.showMoveModal(props.file),
		},
		{
			id: 'download',
			label: ui.downloadButtonLabel ?? formatMessage(commonMessages.downloadButton),
			icon: DownloadIcon,
			action: () => ui.downloadFile(item),
			shown: props.file.type !== 'directory',
		},
		{
			id: 'delete',
			label: formatMessage(commonMessages.deleteLabel),
			icon: TrashIcon,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => ui.showDeleteModal(props.file),
			tone: 'red',
		},
	]
})

const iconStyle = computed(() => fileIconFor(props.file))

const formattedModifiedDate = computed(() => {
	const date = new Date(props.file.modified * 1000)
	return formatDateTime(date)
})

const formattedCreationDate = computed(() => {
	const date = new Date(props.file.created * 1000)
	return formatDateTime(date)
})

const isEditableFile = computed(() => {
	if (props.file.type === 'file') {
		return canOpenInFileEditor(props.file.name)
	}
	return false
})

const columnValues = computed<Record<FileColumn, string | null>>(() => ({
	size: props.file.size != null ? formatBytes(props.file.size) : null,
	items:
		props.file.type === 'directory'
			? formatMessage(messages.itemCount, { count: props.file.count ?? 0 })
			: null,
	created: formattedCreationDate.value,
	modified: formattedModifiedDate.value,
}))

/**
 * Details this row has a value for that aren't visible as columns. Checked against the raw
 * entry, so the formatted values are only computed once the tooltip actually shows.
 */
const hiddenDetails = computed(() => {
	const shown = props.columns ?? []
	return FILE_COLUMNS_ORDER.filter((key) => {
		if (shown.includes(key)) return false
		if (key === 'size') return props.file.size != null
		if (key === 'items') return props.file.type === 'directory'
		return true
	})
})

function openContextMenu(event: MouseEvent) {
	event.preventDefault()
	ui.handleContextMenu(event, menuOptions.value)
}

function handleMouseEnter() {
	hoveringToEdit.value = true
	emit('hover', props.file)
}

function handleMouseLeave() {
	hoveringToEdit.value = false
}

const isNavigating = ref(false)

function selectItem(event?: MouseEvent) {
	if (wasRecentDrag()) return
	if (event?.ctrlKey || event?.metaKey) {
		ui.toggleItemSelection(props.file)
		return
	}
	if (isNavigating.value) return
	isNavigating.value = true

	if (props.file.type === 'directory' || isEditableFile.value) {
		emit('navigate', props.file)
	}

	setTimeout(() => {
		isNavigating.value = false
	}, 500)
}

function handleAuxClick(event: MouseEvent) {
	if (event.button !== 1 || !canOpenInTab.value) return
	event.preventDefault()
	ui.handleOpenInNewTab(props.file)
}

function handleKeydown(event: KeyboardEvent) {
	if (event.key === 'Enter') {
		selectItem()
	} else if (canExpand.value) {
		if (event.key === 'ArrowRight' && !props.expanded) emit('toggle-expand', props.file)
		if (event.key === 'ArrowLeft' && props.expanded) emit('toggle-expand', props.file)
	}
}

function handlePointerDown(e: PointerEvent) {
	if (e.button !== 0) return
	startFileDrag(
		{ name: props.file.name, type: props.file.type, path: props.file.path },
		e,
		(source, destination) => {
			ui.moveItem(source, childPath(destination, source.name))
		},
	)
}
</script>
