<template>
	<slot name="modals" />
	<FileUnsavedChangesModal ref="unsavedChangesModal" />
	<FileCreateItemModal ref="createItemModal" :type="newItemType" @create="handleCreateNewItem" />
	<FileCreateZipModal
		ref="createZipModal"
		:parent="ctx.currentPath.value"
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
		:current-path="ctx.currentPath.value"
		@move="handleMoveItem"
	/>
	<FileDeleteItemModal ref="deleteItemModal" :item="selectedItem" @delete="handleDeleteItem" />
	<ContextMenu ref="contextMenuRef" :label="formatMessage(commonMessages.actionsLabel)" />

	<div ref="fileViewer" v-if="!(ctx.loading.value && items.length === 0)" :class="[!smallMode ? 'h-[50rem]' : '']">
		<KeepAlive>
			<SplitviewVue v-if="!smallMode" class="h-[50rem]"
				:theme="themeDark"
				:orientation="Orientation.HORIZONTAL"
				:components="{ fileSideBar: FileSideBar, fileBrowserPanel: FileBrowserPanel }"
				@ready="onReady"
			/>
		</KeepAlive>
		<template v-if="smallMode">
			<FileBrowserPanel :small-mode="true"/>
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
					v-tooltip="busyTooltip"
					type="quiet"
					:disabled="isBusy"
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
import 'dockview-vue/dist/styles/dockview.css'

import {FolderArchiveIcon, HistoryIcon, SaveIcon, TrashIcon,} from '@modrinth/assets'
import {
	type ISplitviewPanel,
	LayoutPriority,
	Orientation,
	type SplitviewReadyEvent,
	SplitviewVue,
	themeDark
} from 'dockview-vue'
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
import {useFileSearch} from './composables/file-search'
import {useFileSelection} from './composables/file-selection'
import {useFileSorting} from './composables/file-sorting'
import {useFileUndoRedo} from './composables/file-undo-redo'
import type {FileEditorBridge} from './providers/file-browser-ui'
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
const isEditing = computed(() => ctx.editingFile.value !== null)
const isBusy = computed(() => (ctx.isBusy?.value ?? false) || (ctx.isReadOnly?.(ctx.currentPath.value) ?? false),)
const busyTooltip = computed(() => ctx.isReadOnly?.(ctx.currentPath.value) ? ctx.readOnlyReason?.value : ctx.busyTooltip?.value,)

const breadcrumbSegments = computed(() => {
	const path = ctx.currentPath.value
	if (typeof path === 'string') {
		return path.split('/').filter(Boolean)
	}
	return []
})

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
	[...selectedItems.value].some((path) => ctx.isReadOnly?.(path)),
)

const { recordOperation, onKeydown } = useFileUndoRedo(
	(path, newName) => ctx.renameItem(path, newName),
	(source, dest) => ctx.moveItem(source, dest),
	() => ctx.refresh(),
	(title, text, type) => addNotification({ title, text, type }),
)

// Bridge to whichever panel currently hosts the file editor (FileBrowserPanel).
// Dockview mounts that panel itself, so there's no template ref to reach it through -
// it registers its exposed API here on mount instead. See providers/file-browser-ui.ts.
const fileEditorApi = shallowRef<FileEditorBridge | null>(null)

const hasUnsavedChanges = computed(() => fileEditorApi.value?.hasUnsavedChanges?.value ?? false)

async function saveFileContent(exit = false) {
	await fileEditorApi.value?.saveFileContent(exit)
}

function revertChanges() {
	fileEditorApi.value?.revertChanges()
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

const sidebar = ref<ISplitviewPanel | null>(null);
const browser = ref<ISplitviewPanel | null>(null);

// Initialize panels dynamically once the component mounts
function onReady({api}: SplitviewReadyEvent) {
	console.log("Split Ready")

	// Render the left panel
	sidebar.value = api.addPanel({
		index: 0,
		id: 'panel_left',
		component: 'fileSideBar',
		minimumSize: 300, // Initial width in pixels
		size: 300,
		priority: LayoutPriority.Low
	});

	if (!sidebarOpen.value) sidebar.value.api.setVisible(false);

	// Render the right panel
	browser.value = api.addPanel({
		index: 1,
		id: 'panel_right',
		component: 'fileBrowserPanel',
		minimumSize: browserMinSize.value,
		priority: LayoutPriority.High
	});
}

const constrainWidth = computed(() => props.constrainWidth);
const browserMinSize = computed(() => constrainWidth.value ? 700 : 1200)

watch(browserMinSize, (value) => browser.value?.api?.setConstraints({ minimumSize: value }))

const sidebarOpenSetting = useLocalStorage('file-layout-sidebar-open', false, { initOnMounted: true });
const sidebarOpen = ref(sidebarOpenSetting.value);

watch(sidebarOpen, (value) => {
	if(!smallMode.value) {
		sidebar.value?.api?.setVisible(value);
	} else {
		if (value) {
			sidebarModal.value?.show()
		} else {
			sidebarModal.value?.hide()
		}
	}
})

const containerWidth = ref<number>()

useResizeObserver(fileViewer, (entries) => {
	const entry = entries[0]
	containerWidth.value = entry.contentRect.width
})

const smallMode = computed(() => containerWidth.value == null || containerWidth.value < 1100);
const fullWidthSidebar = computed(() => containerWidth.value == null || containerWidth.value < 400); //356

let pastInitialSetup = false;

watch(smallMode, (value) => {
	if (pastInitialSetup) {
		sidebarOpen.value = !value;
	} else {
		pastInitialSetup = true;
	}

	browser.value?.api?.setConstraints({ minimumSize: constrainWidth ? 700 : 1200 })
})

async function confirmDiscardChanges(): Promise<boolean> {
	if (!hasUnsavedChanges.value) return true
	const result = await unsavedChangesModal.value?.prompt()
	if (result === 'save') {
		if (isBusy.value) return false
		await saveFileContent(false)
		return true
	}
	return result === 'discard'
}

// Navigation
async function navigateToSegment(index: number) {
	const newPath = index === -1 ? '/' : breadcrumbSegments.value.slice(0, index + 1).join('/')

	if (newPath === ctx.currentPath.value && !isEditing.value) {
		return
	}

	if (isEditing.value) {
		if (!(await confirmDiscardChanges())) return
		ctx.stopEditing()
	}

	ctx.navigateTo(newPath)
}

function handleNavigateToFolder(item: FileItem) {
	const currentPath = ctx.currentPath.value
	const newPath = currentPath.endsWith('/')
		? `${currentPath}${item.name}`
		: `${currentPath}/${item.name}`
	ctx.navigateTo(newPath)
}

// Editing
function handleEditFile(item: { name: string; type: string; path: string }) {
	ctx.startEditing({ name: item.name, path: item.path })
}

async function handleEditorClose() {
	if (!(await confirmDiscardChanges())) return
	ctx.stopEditing()
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

	const path = `${ctx.currentPath.value}/${item.name}`.replace('//', '/')
	await ctx.renameItem(path, newName)
	recordOperation({
		type: 'rename',
		itemType: item.type,
		fileName: item.name,
		path: ctx.currentPath.value,
		oldName: item.name,
		newName,
	})
}

async function handleMoveItem(destination: string) {
	if (isBusy.value) return
	const item = selectedItem.value
	if (!item) return

	const sourcePath = ctx.currentPath.value
	const source = `${sourcePath}/${item.name}`.replace('//', '/')
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

	const path = `${ctx.currentPath.value}/${item.name}`.replace('//', '/')
	ctx.deleteItem(path, item.type === 'directory')
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
	if (isBusy.value || !ctx.zipPaths || selectedItems.value.size === 0) return
	const include = items.value
		.filter((item) => selectedItems.value.has(item.path))
		.map((item) => item.name)
	deselectAll()
	await ctx.zipPaths(ctx.currentPath.value, include, target)
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

	const itemsToDelete = Array.from(selectedItems.value)
	for (const path of itemsToDelete) {
		const item = items.value.find((i) => i.path === path)
		if (item) {
			ctx.deleteItem(path, item.type === 'directory')
		}
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
			const currentPath = ctx.currentPath.value
			const navPath = currentPath.endsWith('/')
				? `${currentPath}${item.name}`
				: `${currentPath}/${item.name}`
			ctx.prefetchDirectory?.(navPath)
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

// Shared UI state/handlers for the dockview-hosted panels (FileSideBar, FileBrowserPanel).
// Dockview mounts those panels itself via the `components` map + `addPanel()`, so they're
// no longer direct template children - normal prop/emit/ref bindings can't reach them. They
// stay true descendants in the Vue tree though (dockview-vue teleports them), so provide/inject
// still works and is what they use instead. See providers/file-browser-ui.ts.
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
	fileEditorApi,
	hasUnsavedChanges,
	saveFileContent,
	revertChanges,
	shareToMclogs,
	toggleFind,

	navigateToSegment,
	handleNavigateToFolder,
	handleEditFile,
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

// Reset search/sort/selection on path change
watch(
	() => ctx.currentPath.value,
	() => {
		searchQuery.value = ''
		resetSort()
		deselectAll()
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
