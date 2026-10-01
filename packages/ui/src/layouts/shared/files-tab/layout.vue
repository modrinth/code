<template>
	<slot name="modals" />
	<FileUnsavedChangesModal ref="unsavedChangesModal" />
	<FileCreateItemModal ref="createItemModal" :type="newItemType" @create="handleCreateNewItem" />
	<FileCreateZipModal
		ref="createZipModal"
		:parent="selectionParent ?? ctx.currentPath.value"
		:stat-file="ctx.statFile"
		@create="handleZipSelection"
	/>
	<FileUploadConflictModal ref="uploadConflictModal" @proceed="handleExtractConfirm" />
	<FileUploadZipUrlModal
		v-if="ctx.showInstallFromUrl"
		ref="uploadZipUrlModal"
		:disabled="isBusy"
		:disabled-tooltip="busyTooltip"
	/>
	<FileRenameItemModal ref="renameItemModal" :item="selectedItem" @rename="handleRenameItem" />
	<FileMoveItemModal
		ref="moveItemModal"
		:item="selectedItem"
		:current-path="selectedItem ? parentDirectory(normalizeFilePath(selectedItem.path)) : ctx.currentPath.value"
		@move="handleMoveItem"
	/>
	<FileDeleteItemModal ref="deleteItemModal" :item="selectedItem" @delete="handleDeleteItem" />
	<ContextMenu ref="contextMenuRef" :label="formatMessage(commonMessages.actionsLabel)" />

	<div v-if="hasLoadedOnce" ref="fileViewer">
		<div
			class="grid items-start"
			:class="{ 'cursor-col-resize select-none': isResizingSidebar }"
			:style="{
				gridTemplateColumns: showDockedSidebar
					? `${sidebarWidth}px auto minmax(0, 1fr)`
					: 'minmax(0, 1fr)',
			}"
		>
			<template v-if="showDockedSidebar">
				<aside
					class="sticky top-[var(--files-sticky-top,0px)] flex max-h-[calc(var(--files-viewport-height,100dvh)_-_var(--files-sticky-top,0px))] flex-col"
				>
					<FileSideBar class="min-h-0" fill-height :add-border="false" />
				</aside>
				<div
					role="separator"
					tabindex="0"
					aria-orientation="vertical"
					:aria-label="formatMessage(messages.resizeSidebar)"
					:aria-valuenow="sidebarWidth"
					:aria-valuemin="SIDEBAR_MIN_WIDTH"
					:aria-valuemax="maxSidebarWidth"
					class="group flex w-3 cursor-col-resize touch-none justify-center self-stretch focus-visible:outline-none"
					@pointerdown="startSidebarResize"
					@keydown.left.prevent="nudgeSidebarWidth(-16)"
					@keydown.right.prevent="nudgeSidebarWidth(16)"
					@dblclick="resetSidebarWidth"
				>
					<div
						class="h-full w-px bg-surface-5 transition-colors group-hover:bg-brand group-focus-visible:bg-brand"
						:class="{ '!bg-brand': isResizingSidebar }"
					/>
				</div>
			</template>
			<div ref="mainColumn" class="min-w-0">
				<FileBrowserPanel :small-mode="smallMode" />
			</div>
		</div>
		<template v-if="smallMode">
			<NewModal ref="sidebarModal"
				:on-hide="() => {
					if (smallMode) sidebarOpen = false;
				}"
				:noblur="true"
				:no-padding="true"
				:hideHeader="true"
				:fill-width-when-small="false"
				:maxWidth="fullWidthSidebar ? '100dvw' : 'fit-content'"
				:max-width-min-check="false"
				:scrollable="false"
				pullout-direction="left"
			>
				<div class="p-2 h-full " :class="[fullWidthSidebar ? 'w-full' : '']">
					<FileSideBar :constrain-width="!fullWidthSidebar" :scroll-file-entries="true" :add-border="false"/>
				</div>
			</NewModal>
		</template>

		<FloatingActionBar :shown="hasUnsavedChanges">
			<p class="m-0 text-sm font-semibold md:text-base">
				{{ formatMessage(messages.unsavedChanges) }}
			</p>
			<div class="ml-auto flex gap-2">
				<Button type="quiet" @click="revertChanges()">
					<HistoryIcon /> {{ formatMessage(commonMessages.resetButton) }}
				</Button>
				<Button
					v-tooltip="isBusy ? busyTooltip : undefined"
					type="colored"
					color="brand"
					:disabled="isBusy"
					@click="saveFileContent(false)"
				>
					<SaveIcon /> {{ formatMessage(commonMessages.saveButton) }}
				</Button>
			</div>
		</FloatingActionBar>
		<FloatingActionBar :shown="selectedItems.size > 0">
			<div class="flex items-center gap-0.5">
				<span class="px-4 py-2.5 text-base font-semibold text-contrast tabular-nums">
					{{ formatMessage(messages.selectedCount, { count: selectedItems.size }) }}
				</span>
				<div class="mx-1 h-6 w-px bg-surface-5" />
				<Button type="quiet" class="!text-primary" @click="deselectAll">
					<span class="bar-label">{{ formatMessage(commonMessages.clearButton) }}</span>
				</Button>
			</div>
			<div class="ml-auto flex items-center gap-0.5">
				<Button
					v-if="ctx.zipPaths"
					v-tooltip="isBusy ? busyTooltip : selectionParent === null ? formatMessage(messages.zipMixedFolders) : undefined"
					type="quiet"
					:disabled="isBusy || selectionParent === null"
					@click="createZipModal?.show()"
				>
					<FolderArchiveIcon />
					<span class="bar-label">{{ formatMessage(messages.createZip) }}</span>
				</Button>
				<div class="mx-1 h-6 w-px bg-surface-5" />
				<Button
					v-tooltip="busyTooltip"
					type="quiet"
					color="red"
					:disabled="isBusy || selectionReadOnly"
					class="hover:!bg-red focus-visible:!bg-red hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
					@click="showBulkDeleteModal"
				>
					<TrashIcon />
					<span class="bar-label">{{ formatMessage(commonMessages.deleteLabel) }}</span>
				</Button>
			</div>
		</FloatingActionBar>
	</div>
</template>

<script setup lang="ts">
import {FolderArchiveIcon, HistoryIcon, SaveIcon, TrashIcon,} from '@modrinth/assets'
import type {Component} from 'vue'
import {computed, onMounted, onUnmounted, ref, shallowRef, watch} from 'vue'

import {type ButtonMenuOption, NewModal} from '#ui/components'
import {Button, ContextMenu} from '#ui/components/base/buttons'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import {defineMessages, useVIntl} from '#ui/composables/i18n'
import {injectFilePicker} from '#ui/providers/file-picker'
import {injectNotificationManager} from '#ui/providers/web-notifications'
import {commonMessages} from '#ui/utils/common-messages'
import {canOpenInFileEditor} from '#ui/utils/file-extensions'

import FileBrowserPanel from './components/FileBrowserPanel.vue'
import FileSideBar from './components/FileSideBar.vue'
import FileCreateItemModal from './components/modals/FileCreateItemModal.vue'
import FileCreateZipModal from './components/modals/FileCreateZipModal.vue'
import FileDeleteItemModal from './components/modals/FileDeleteItemModal.vue'
import FileMoveItemModal from './components/modals/FileMoveItemModal.vue'
import FileRenameItemModal from './components/modals/FileRenameItemModal.vue'
import FileUnsavedChangesModal from './components/modals/FileUnsavedChangesModal.vue'
import FileUploadConflictModal from './components/modals/FileUploadConflictModal.vue'
import FileUploadZipUrlModal from './components/modals/FileUploadZipUrlModal.vue'
import {
	directoryOf,
	type FileLocation,
	normalizeFilePath,
	parentDirectory,
	useFileTabs,
} from './composables/file-tabs'
import {useFileSearch} from './composables/file-search'
import {useFileSelection} from './composables/file-selection'
import {useFileSorting} from './composables/file-sorting'
import {useFileUndoRedo} from './composables/file-undo-redo'
import type {FileEditorBridge, FileInfo} from './providers/file-browser-ui'
import {provideFileBrowserUI} from './providers/file-browser-ui'
import {injectFileManager} from './providers/file-manager'
import type {FileItem} from './types'
import {type MaybeElement, useLocalStorage, useResizeObserver} from "@vueuse/core";

const { formatMessage } = useVIntl()

const messages = defineMessages({
	emptyFolderTitle: {
		id: 'files.layout.empty-folder-title',
		defaultMessage: 'This folder is empty',
	},
	emptyFolderDescription: {
		id: 'files.layout.empty-folder-description',
		defaultMessage: 'There are no files or folders.',
	},
	errorTitle: {
		id: 'files.layout.error-title',
		defaultMessage: 'Unable to load files',
	},
	errorMessage: {
		id: 'files.layout.error-message',
		defaultMessage: 'The folder may not exist.',
	},
	selectedCount: {
		id: 'files.layout.selected-count',
		defaultMessage: '{count} selected',
	},
	dryRunFailedTitle: {
		id: 'files.layout.dry-run-failed-title',
		defaultMessage: 'Dry run failed',
	},
	dryRunFailedText: {
		id: 'files.layout.dry-run-failed-text',
		defaultMessage: 'Error running dry run',
	},
	extractionStartedTitle: {
		id: 'files.layout.extraction-started-title',
		defaultMessage: 'Extraction started',
	},
	unsavedChanges: {
		id: 'files.layout.unsaved-changes',
		defaultMessage: 'You have unsaved changes.',
	},
	createZip: {
		id: 'files.layout.create-zip',
		defaultMessage: 'Create ZIP',
	},
	resizeSidebar: {
		id: 'files.layout.resize-sidebar',
		defaultMessage: 'Resize sidebar',
	},
	zipMixedFolders: {
		id: 'files.layout.zip-mixed-folders',
		defaultMessage: 'Only entries from the same folder can be zipped together',
	},
})

const props = defineProps<{
	showDebugInfo?: boolean
	showRefreshButton?: boolean
	constrainWidth?: boolean
}>()

const { addNotification } = injectNotificationManager()
const ctx = injectFileManager()
const filePicker = injectFilePicker(null)

const editorComponent = shallowRef<Component | null>(null)
import('vue3-ace-editor').then(async (mod) => {
	await Promise.all([import('#ui/utils/ace-theme'), import('#ui/utils/ace-mode-log.ts')])
	editorComponent.value = mod.VAceEditor
})

const baseId = `files-${Math.random().toString(36).slice(2, 9)}`

const items = computed(() => ctx.currentItems.value)

/**
 * Only the very first load hides the viewer. Unmounting it on later navigations would tear down
 * the tabs' dockview, and with it every other tab's open editor.
 */
const hasLoadedOnce = ref(false)
watch(
	() => !(ctx.loading.value && items.value.length === 0),
	(loaded) => {
		if (loaded) hasLoadedOnce.value = true
	},
	{ immediate: true },
)
const isEditing = computed(() => ctx.editingFile.value !== null)
const isBusy = computed(() => (ctx.isBusy?.value ?? false) || (ctx.isReadOnly?.(ctx.currentPath.value) ?? false),)
const busyTooltip = computed(() => ctx.isReadOnly?.(ctx.currentPath.value) ? ctx.readOnlyReason?.value : ctx.busyTooltip?.value,)

const breadcrumbSegments = computed(() =>
	directoryOf(fileTabs.activeLocation.value).split('/').filter(Boolean),
)

// Composables
const { searchQuery, searchedItems } = useFileSearch(items)
const {
	sortField,
	sortDesc,
	handleSort,
	sortedItems: filteredItems,
	resetSort,
} = useFileSorting(searchedItems)

const {
	selectedItems,
	toggleItemSelection,
	deselectAll,
	toggleSelectAll,
	allSelected,
	someSelected,
} = useFileSelection(filteredItems)

const selectionReadOnly = computed(() =>
	[...selectedItems.value.keys()].some((path) => ctx.isReadOnly?.(path)),
)

/** The directory all selected entries share, or `null` when the selection is empty or spans directories. */
const selectionParent = computed(() => {
	const parents = new Set(
		[...selectedItems.value.values()].map((item) => parentDirectory(normalizeFilePath(item.path))),
	)
	return parents.size === 1 ? [...parents][0] : null
})

const { recordOperation, onKeydown } = useFileUndoRedo(
	(path, newName) => ctx.renameItem(path, newName),
	(source, dest) => ctx.moveItem(source, dest),
	() => ctx.refresh(),
	(title, text, type) => addNotification({ title, text, type }),
)

const fileTabs = useFileTabs({
	ctx,
	confirmDiscard: confirmDiscardEditors,
})

const fileEditorApi = fileTabs.activeEditor
const hasUnsavedChanges = fileTabs.hasUnsavedChanges

function dirtyEditors(editors: Iterable<FileEditorBridge> = fileTabs.editors.values()) {
	return [...editors].filter((editor) => editor.hasUnsavedChanges.value)
}

async function saveFileContent(exit = false) {
	await Promise.all(dirtyEditors().map((editor) => editor.saveFileContent(exit)))
}

function revertChanges() {
	for (const editor of dirtyEditors()) editor.revertChanges()
}

async function shareToMclogs() {
	await fileEditorApi.value?.shareToMclogs()
}

function toggleFind() {
	fileEditorApi.value?.toggleFind()
}

// Refs
const fileViewer = ref<MaybeElement>();
const sidebarModal = ref<InstanceType<typeof NewModal>>();
const createItemModal = ref<InstanceType<typeof FileCreateItemModal>>()
const createZipModal = ref<InstanceType<typeof FileCreateZipModal>>()
const renameItemModal = ref<InstanceType<typeof FileRenameItemModal>>()
const moveItemModal = ref<InstanceType<typeof FileMoveItemModal>>()
const deleteItemModal = ref<InstanceType<typeof FileDeleteItemModal>>()
const uploadConflictModal = ref<InstanceType<typeof FileUploadConflictModal>>()
const uploadZipUrlModal = ref<InstanceType<typeof FileUploadZipUrlModal>>()
const contextMenuRef = ref<InstanceType<typeof ContextMenu>>()

const newItemType = ref<'file' | 'directory'>('file')
const selectedItem = ref<FileItem | null>(null)

const unsavedChangesModal = ref<InstanceType<typeof FileUnsavedChangesModal>>()

const mainColumn = ref<MaybeElement>()

const SIDEBAR_MIN_WIDTH = 240
const SIDEBAR_DEFAULT_WIDTH = 300

const browserMinSize = computed(() => (props.constrainWidth ? 700 : 1200))

const sidebarOpenSetting = useLocalStorage('file-layout-sidebar-open', false, { initOnMounted: true });
const sidebarOpen = ref(sidebarOpenSetting.value);
const sidebarWidthSetting = useLocalStorage('file-layout-sidebar-width', SIDEBAR_DEFAULT_WIDTH, {
	initOnMounted: true,
})

watch(sidebarOpen, (value) => {
	if (!smallMode.value) return
	if (value) {
		sidebarModal.value?.show()
	} else {
		sidebarModal.value?.hide()
	}
})

/** Width of the whole files tab, used to pick between the docked sidebar and the pullout modal. */
const shellWidth = ref<number>()
/** Width of the main column, i.e. what `FileBrowserPanel` actually gets next to the sidebar. */
const containerWidth = ref<number>()

useResizeObserver(fileViewer, (entries) => {
	shellWidth.value = entries[0].contentRect.width
})

useResizeObserver(mainColumn, (entries) => {
	containerWidth.value = entries[0].contentRect.width
})

const smallMode = computed(() => shellWidth.value == null || shellWidth.value < 1100);
const fullWidthSidebar = computed(() => shellWidth.value == null || shellWidth.value < 400);
const showDockedSidebar = computed(() => !smallMode.value && sidebarOpen.value)

const maxSidebarWidth = computed(() =>
	Math.max(SIDEBAR_MIN_WIDTH, (shellWidth.value ?? 0) - browserMinSize.value),
)

function clampSidebarWidth(width: number) {
	return Math.round(Math.min(Math.max(width, SIDEBAR_MIN_WIDTH), maxSidebarWidth.value))
}

const sidebarWidth = computed(() => clampSidebarWidth(sidebarWidthSetting.value))
const isResizingSidebar = ref(false)

function startSidebarResize(event: PointerEvent) {
	if (event.button !== 0) return
	event.preventDefault()

	const handle = event.currentTarget as HTMLElement
	const startX = event.clientX
	const startWidth = sidebarWidth.value

	handle.setPointerCapture(event.pointerId)
	isResizingSidebar.value = true

	function onMove(moveEvent: PointerEvent) {
		sidebarWidthSetting.value = clampSidebarWidth(startWidth + moveEvent.clientX - startX)
	}

	function onEnd() {
		isResizingSidebar.value = false
		handle.removeEventListener('pointermove', onMove)
		handle.removeEventListener('pointerup', onEnd)
		handle.removeEventListener('pointercancel', onEnd)
	}

	handle.addEventListener('pointermove', onMove)
	handle.addEventListener('pointerup', onEnd)
	handle.addEventListener('pointercancel', onEnd)
}

function nudgeSidebarWidth(delta: number) {
	sidebarWidthSetting.value = clampSidebarWidth(sidebarWidth.value + delta)
}

function resetSidebarWidth() {
	sidebarWidthSetting.value = SIDEBAR_DEFAULT_WIDTH
}

let pastInitialSetup = false;

watch(smallMode, (value) => {
	if (pastInitialSetup) {
		sidebarOpen.value = !value;
	} else {
		pastInitialSetup = true;
	}
})

/**
 * Prompts about any unsaved changes in the given editors.
 * Resolves `true` once it is safe to discard them (saved, discarded, or nothing to lose).
 */
async function confirmDiscardEditors(editors: Iterable<FileEditorBridge>): Promise<boolean> {
	const dirty = dirtyEditors(editors)
	if (dirty.length === 0) return true

	const result = await unsavedChangesModal.value?.prompt()
	if (result === 'save') {
		if (isBusy.value) return false
		await Promise.all(dirty.map((editor) => editor.saveFileContent(false)))
		return dirty.every((editor) => !editor.hasUnsavedChanges.value)
	}
	return result === 'discard'
}

function locationOf(item: FileInfo): FileLocation {
	const path = normalizeFilePath(item.path)
	return item.type === 'directory'
		? { kind: 'directory', path }
		: { kind: 'file', path, name: item.name }
}

async function navigateToSegment(index: number) {
	const path = `/${breadcrumbSegments.value.slice(0, index + 1).join('/')}`
	await fileTabs.navigate({ kind: 'directory', path })
}

async function handleNavigateToFolder(item: FileInfo) {
	await fileTabs.navigate(locationOf({ ...item, type: 'directory' }))
}

async function handleEditFile(item: FileInfo) {
	await fileTabs.navigate(locationOf({ ...item, type: 'file' }))
}

function handleOpenInNewTab(item: FileInfo) {
	if (item.type !== 'directory' && !canOpenInFileEditor(item.name)) return
	fileTabs.openTab(locationOf(item))
}

async function handleEditorClose() {
	const location = fileTabs.activeLocation.value
	if (location.kind !== 'file') return
	await fileTabs.navigate({ kind: 'directory', path: directoryOf(location) })
}

// CRUD handlers
async function handleCreateNewItem(name: string) {
	if (isBusy.value) return
	await ctx.createItem(name, newItemType.value)
}

async function handleRenameItem(newName: string) {
	if (isBusy.value) return
	const item = selectedItem.value
	if (!item) return

	const path = normalizeFilePath(item.path)
	await ctx.renameItem(path, newName)
	recordOperation({
		type: 'rename',
		itemType: item.type,
		fileName: item.name,
		path: parentDirectory(path),
		oldName: item.name,
		newName,
	})
}

async function handleMoveItem(destination: string) {
	if (isBusy.value) return
	const item = selectedItem.value
	if (!item) return

	const source = normalizeFilePath(item.path)
	const sourcePath = parentDirectory(source)
	const dest = `${destination}/${item.name}`.replace('//', '/')

	await ctx.moveItem(source, dest)
	recordOperation({
		type: 'move',
		sourcePath,
		destinationPath: destination,
		fileName: item.name,
		itemType: item.type,
	})
}

function handleDeleteItem() {
	if (isBusy.value) return
	const item = selectedItem.value
	if (!item) return

	ctx.deleteItem(normalizeFilePath(item.path), item.type === 'directory')
}

function handleDirectMove(moveData: {
	name: string
	type: string
	path: string
	destination: string
}) {
	if (isBusy.value) return
	const dest = `${moveData.destination}/${moveData.name}`.replace('//', '/')
	const sourcePath = moveData.path.substring(0, moveData.path.lastIndexOf('/'))

	ctx.moveItem(moveData.path, dest).then(() => {
		recordOperation({
			type: 'move',
			sourcePath,
			destinationPath: moveData.destination,
			fileName: moveData.name,
			itemType: moveData.type,
		})
	})
}

// Download
async function handleDownload(item: FileItem) {
	if (item.type === 'file') {
		await ctx.downloadFile(item.path, item.name)
	}
}

async function handleZip(item: FileItem) {
	if (isBusy.value || item.type !== 'directory' || !ctx.zipFolder) return
	await ctx.zipFolder(item.path)
}

async function handleZipSelection(target: string) {
	const parent = selectionParent.value
	if (isBusy.value || !ctx.zipPaths || parent === null) return
	const include = [...selectedItems.value.values()].map((item) => item.name)
	deselectAll()
	await ctx.zipPaths(parent, include, target)
}

// Extract
async function handleExtractItem(item: { name: string; type: string; path: string }) {
	if (isBusy.value || !ctx.extractFile) return
	try {
		const dry = await ctx.extractFile(item.path, true, true)
		if (dry) {
			if (dry.conflicting_files.length === 0) {
				handleExtractConfirm(item.path)
			} else {
				uploadConflictModal.value?.show(item.path, dry.conflicting_files)
			}
		} else {
			addNotification({
				title: formatMessage(messages.dryRunFailedTitle),
				text: formatMessage(messages.dryRunFailedText),
				type: 'error',
			})
		}
	} catch (error) {
		addNotification({
			title: formatMessage(commonMessages.extractFailedLabel),
			text: error instanceof Error ? error.message : '',
			type: 'error',
		})
	}
}

async function handleExtractConfirm(path: string) {
	if (isBusy.value) return
	if (!ctx.extractFile) return
	try {
		await ctx.extractFile(path, true, false)
		addNotification({ title: formatMessage(messages.extractionStartedTitle), type: 'success' })
	} catch (error) {
		addNotification({
			title: formatMessage(commonMessages.extractFailedLabel),
			text: error instanceof Error ? error.message : '',
			type: 'error',
		})
	}
}

// Modal show helpers
function showCreateModal(type: 'file' | 'directory') {
	if (isBusy.value) return
	newItemType.value = type
	createItemModal.value?.show()
}

function showUnzipFromUrlModal(cf: boolean) {
	if (isBusy.value) return
	uploadZipUrlModal.value?.show(cf)
}

function showRenameModal(item: FileItem) {
	if (isBusy.value || ctx.isReadOnly?.(item.path)) return
	selectedItem.value = item
	renameItemModal.value?.show(item)
}

function showMoveModal(item: FileItem) {
	if (isBusy.value || ctx.isReadOnly?.(item.path)) return
	selectedItem.value = item
	moveItemModal.value?.show()
}

function showDeleteModal(item: FileItem) {
	if (isBusy.value || ctx.isReadOnly?.(item.path)) return
	selectedItem.value = item
	deleteItemModal.value?.show()
}

function showBulkDeleteModal() {
	if (isBusy.value || selectionReadOnly.value) return
	if (selectedItems.value.size === 0) return

	for (const item of selectedItems.value.values()) {
		ctx.deleteItem(normalizeFilePath(item.path), item.type === 'directory')
	}
	deselectAll()
}

// Upload
function handleDroppedFiles(files: File[]) {
	if (isEditing.value || isBusy.value) return
	ctx.uploadFiles(files)
}

function handleDropError(error: unknown) {
	addNotification({
		title: formatMessage(commonMessages.uploadFailedLabel),
		text: error instanceof Error ? error.message : undefined,
		type: 'error',
	})
}

async function initiateFileUpload() {
	if (isBusy.value) return
	if (filePicker?.pickFiles) {
		try {
			const picked = await filePicker.pickFiles({ multiple: true })
			if (picked.length > 0) {
				ctx.uploadFiles(picked.map((item) => item.file))
			}
		} catch (error) {
			addNotification({
				title: formatMessage(commonMessages.uploadFailedLabel),
				text: error instanceof Error ? error.message : undefined,
				type: 'error',
			})
		}
		return
	}

	const input = document.createElement('input')
	input.type = 'file'
	input.multiple = true
	input.onchange = () => {
		if (input.files) {
			ctx.uploadFiles(Array.from(input.files))
		}
	}
	input.click()
}

// Prefetch
let prefetchTimeout: ReturnType<typeof setTimeout> | null = null
let prefetchHomeTimeout: ReturnType<typeof setTimeout> | null = null

function handleItemHover(item: { type: string; path: string; name: string }) {
	if (prefetchTimeout) {
		clearTimeout(prefetchTimeout)
		prefetchTimeout = null
	}

	if (item.type === 'directory') {
		prefetchTimeout = setTimeout(() => {
			ctx.prefetchDirectory?.(normalizeFilePath(item.path))
		}, 150)
	} else if (canOpenInFileEditor(item.name)) {
		prefetchTimeout = setTimeout(() => {
			ctx.prefetchFile?.(item.path)
		}, 150)
	}
}

function handlePrefetchHome() {
	if (prefetchHomeTimeout) {
		clearTimeout(prefetchHomeTimeout)
		prefetchHomeTimeout = null
	}
	prefetchHomeTimeout = setTimeout(() => {
		ctx.prefetchDirectory?.('/')
	}, 150)
}

function handleContextMenu(event: MouseEvent, options: ButtonMenuOption[]) {
	contextMenuRef.value?.open(event, options)
}

// Shared UI state/handlers for FileSideBar, FileBrowserPanel and the dockview-hosted editor
// tabs. Dockview mounts tab panels itself (teleported, so still Vue descendants), which means
// prop/emit/ref bindings can't reach them but provide/inject does. See providers/file-browser-ui.ts.
provideFileBrowserUI({
	baseId,
	showDebugInfo: computed(() => props.showDebugInfo ?? false),
	showRefreshButton: computed(() => props.showRefreshButton ?? false),

	items,
	filteredItems,
	isEditing,
	isBusy,
	busyTooltip,
	breadcrumbSegments,
	sidebarOpen: computed(() => sidebarOpen.value),
	setSidebarOpen: (value) => {
		sidebarOpen.value = value;
		sidebarOpenSetting.value = value;
	},
	containerWidth,

	searchQuery,
	sortField,
	sortDesc,
	handleSort,

	selectedItems,
	toggleItemSelection,
	deselectAll,
	toggleSelectAll,
	allSelected,
	someSelected,

	editorComponent,
	fileTabs,
	fileEditorApi,
	hasUnsavedChanges,
	saveFileContent,
	revertChanges,
	shareToMclogs,
	toggleFind,

	navigateToSegment,
	handleNavigateToFolder,
	handleEditFile,
	handleOpenInNewTab,
	handleEditorClose,
	handlePrefetchHome,
	handleItemHover,

	showCreateModal,
	showRenameModal,
	showMoveModal,
	showDeleteModal,
	showBulkDeleteModal,
	showUnzipFromUrlModal,

	handleDownload,
	handleZip,
	handleDirectMove,
	handleExtractItem,

	handleDroppedFiles,
	handleDropError,
	initiateFileUpload,

	handleContextMenu,
})

// Reset search/sort on path change; selection spans directories, so it is kept
watch(
	() => ctx.currentPath.value,
	() => {
		searchQuery.value = ''
		resetSort()
	},
)

// Keyboard shortcuts
onMounted(() => {
	document.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
	document.removeEventListener('keydown', onKeydown)
})
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
	transition:
		opacity 300ms ease-in-out,
		transform 300ms ease-in-out;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
	transform: scale(0.98);
}
</style>
