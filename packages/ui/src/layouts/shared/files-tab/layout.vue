<template>
	<slot name="modals" />
	<FileCreateItemModal ref="createItemModal" :type="newItemType" @create="handleCreateNewItem" />
	<FileUploadConflictModal ref="uploadConflictModal" @proceed="handleExtractConfirm" />
	<FileUploadZipUrlModal ref="uploadZipUrlModal" />
	<FileRenameItemModal ref="renameItemModal" :item="selectedItem" @rename="handleRenameItem" />
	<FileMoveItemModal
		ref="moveItemModal"
		:item="selectedItem"
		:current-path="ctx.currentPath.value"
		@move="handleMoveItem"
	/>
	<FileDeleteItemModal ref="deleteItemModal" :item="selectedItem" @delete="handleDeleteItem" />
	<FileContextMenu ref="contextMenuRef">
		<template #extract><PackageOpenIcon class="size-5" /> Extract</template>
		<template #rename><EditIcon class="size-5" /> Rename</template>
		<template #move><RightArrowIcon class="size-5" /> Move</template>
		<template #download><DownloadIcon class="size-5" /> Download</template>
		<template #delete><TrashIcon class="size-5" /> Delete</template>
	</FileContextMenu>
	<Transition name="fade" mode="out-in">
		<div
			v-if="ctx.loading.value && items.length === 0"
			key="loading"
			class="mt-6 flex flex-col items-center justify-center gap-2 text-center text-secondary"
		>
			<SpinnerIcon class="animate-spin" />
			Loading files...
		</div>

		<div v-else key="content" class="contents">
			<Admonition v-if="ctx.busyWarning?.value" type="warning" class="mb-5">
				<template #header>{{ ctx.busyWarning.value }}</template>
				File operations are disabled while the operation is in progress.
			</Admonition>
			<Transition
				enter-active-class="transition-all duration-300 ease-out overflow-hidden"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-40"
				leave-active-class="transition-all duration-200 ease-in overflow-hidden"
				leave-from-class="opacity-100 max-h-40"
				leave-to-class="opacity-0 max-h-0"
			>
				<Admonition
					v-if="ctx.uploadState?.value?.isUploading"
					type="info"
					class="mb-4"
					show-actions-underneath
				>
					<template #icon>
						<UploadIcon class="h-6 w-6 flex-none text-brand-blue" />
					</template>
					<template #header>
						Uploading files ({{ ctx.uploadState.value.completedFiles }}/{{
							ctx.uploadState.value.totalFiles
						}})
						<span v-if="ctx.uploadState.value.currentFileName" class="font-normal text-secondary">
							— {{ ctx.uploadState.value.currentFileName }}
						</span>
					</template>
					<span class="text-secondary">
						{{ formatBytes(ctx.uploadState.value.uploadedBytes) }}
						/ {{ formatBytes(ctx.uploadState.value.totalBytes) }} ({{
							Math.round(uploadOverallProgress * 100)
						}}%)
					</span>
					<template #actions>
						<ProgressBar :progress="uploadOverallProgress" :max="1" color="blue" full-width />
					</template>
				</Admonition>
			</Transition>
			<Admonition
				v-for="op in activeOperations"
				:key="`fs-op-${op.op}-${op.src}`"
				:type="op.state === 'done' ? 'success' : op.state?.startsWith('fail') ? 'error' : 'info'"
				class="mb-4"
				show-actions-underneath
			>
				<template #icon>
					<PackageOpenIcon class="h-6 w-6 flex-none text-brand-blue" />
				</template>
				<template #header>
					Extracting {{ op.src.includes('https://') ? 'modpack from URL' : op.src }}
					<span v-if="op.state === 'done'" class="font-normal text-green"> — Done</span>
					<span v-else-if="op.state?.startsWith('fail')" class="font-normal text-red">
						— Failed</span
					>
				</template>
				<span class="text-secondary">
					{{ 'bytes_processed' in op ? formatBytes(op.bytes_processed ?? 0) : '0 B' }} extracted
					<template v-if="'current_file' in op && op.current_file">
						— {{ op.current_file?.split('/')?.pop() }}
					</template>
				</span>
				<template #actions>
					<ProgressBar
						:progress="'progress' in op ? (op.progress ?? 0) : 0"
						:max="1"
						:color="op.state === 'done' ? 'green' : op.state?.startsWith('fail') ? 'red' : 'blue'"
						:waiting="op.state === 'queued' || !op.progress || op.progress === 0"
						full-width
					/>
				</template>
			</Admonition>
			<div class="relative flex w-full flex-col">
				<div class="relative isolate flex w-full flex-col gap-2">
					<FileNavbar
						:breadcrumbs="breadcrumbSegments"
						:is-editing="isEditing"
						:editing-file-name="ctx.editingFile.value?.name"
						:editing-file-path="ctx.editingFile.value?.path"
						:is-editing-image="fileEditorRef?.isEditingImage"
						:search-query="searchQuery"
						:show-refresh-button="showRefreshButton"
						:base-id="baseId"
						:disabled="isBusy"
						:disabled-tooltip="busyTooltip"
						@navigate="navigateToSegment"
						@navigate-home="() => navigateToSegment(-1)"
						@prefetch-home="handlePrefetchHome"
						@update:search-query="searchQuery = $event"
						@create="showCreateModal"
						@upload="initiateFileUpload"
						@upload-zip="() => {}"
						@unzip-from-url="showUnzipFromUrlModal"
						@refresh="ctx.refresh"
						@save="() => fileEditorRef?.saveFileContent(true)"
						@save-as="() => fileEditorRef?.saveFileContent(false)"
						@save-restart="() => fileEditorRef?.saveAndRestart()"
						@share="() => fileEditorRef?.shareToMclogs()"
					/>

					<div v-if="!isEditing" class="contents">
						<FileUploadDragAndDrop
							ref="fileUploadRef"
							class="relative flex flex-col overflow-clip rounded-[20px] border border-solid border-surface-4 shadow-sm"
							@files-dropped="handleDroppedFiles"
						>
							<FileTableHeader
								:sort-field="sortField"
								:sort-desc="sortDescValue"
								:all-selected="allSelected"
								:some-selected="someSelected"
								:is-stuck="isLabelBarStuck"
								@sort="handleSort"
								@toggle-all="toggleSelectAll"
							/>
							<Transition name="fade" mode="out-in">
								<div v-if="items.length > 0" key="list" class="h-full w-full overflow-hidden">
									<FileVirtualList
										:items="filteredItems"
										:selected-items="selectedItems"
										:write-disabled="isBusy"
										:write-disabled-tooltip="busyTooltip"
										@extract="handleExtractItem"
										@delete="showDeleteModal"
										@rename="showRenameModal"
										@download="handleDownload"
										@move="showMoveModal"
										@move-direct-to="handleDirectMove"
										@edit="handleEditFile"
										@navigate="handleNavigateToFolder"
										@hover="handleItemHover"
										@contextmenu="handleContextMenu"
										@toggle-select="toggleItemSelection"
									/>
								</div>
								<div
									v-else-if="items.length === 0 && !ctx.error.value"
									key="empty"
									class="flex h-full w-full items-center justify-center rounded-b-[20px] bg-surface-2 p-20"
								>
									<div class="flex flex-col items-center gap-4 text-center">
										<FolderOpenIcon class="h-16 w-16 text-secondary" />
										<h3 class="m-0 text-2xl font-bold text-contrast">This folder is empty</h3>
										<p class="m-0 text-sm text-secondary">There are no files or folders.</p>
									</div>
								</div>
								<FileManagerError
									v-else-if="ctx.error.value"
									key="error"
									class="rounded-b-[20px]"
									title="Unable to load files"
									message="The folder may not exist."
									@refetch="ctx.refresh"
									@home="navigateToSegment(-1)"
								/>
							</Transition>
						</FileUploadDragAndDrop>
					</div>
					<FileEditor
						v-else
						ref="fileEditorRef"
						:file="ctx.editingFile.value"
						:editor-component="ctx.editorComponent.value"
						@close="handleEditorClose"
					/>
				</div>
			</div>

			<FloatingActionBar :shown="selectedItems.size > 0">
				<div class="flex items-center gap-0.5">
					<span class="px-4 py-2.5 text-base font-semibold text-contrast tabular-nums">
						{{ selectedItems.size }} selected
					</span>
					<div class="mx-1 h-6 w-px bg-surface-5" />
					<ButtonStyled type="transparent">
						<button class="!text-primary" @click="deselectAll">
							<XIcon />
							<span class="bar-label">Clear</span>
						</button>
					</ButtonStyled>
				</div>
				<div class="ml-auto flex items-center gap-0.5">
					<div class="mx-1 h-6 w-px bg-surface-5" />
					<ButtonStyled
						type="transparent"
						color="red"
						color-fill="text"
						hover-color-fill="background"
					>
						<button v-tooltip="busyTooltip" :disabled="isBusy" @click="showBulkDeleteModal">
							<TrashIcon />
							<span class="bar-label">Delete</span>
						</button>
					</ButtonStyled>
				</div>
			</FloatingActionBar>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import {
	DownloadIcon,
	EditIcon,
	FolderOpenIcon,
	PackageOpenIcon,
	RightArrowIcon,
	SpinnerIcon,
	TrashIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import { formatBytes } from '@modrinth/utils'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import ProgressBar from '#ui/components/base/ProgressBar.vue'
import { useStickyObserver } from '#ui/composables/sticky-observer'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { getFileExtension } from '#ui/utils/file-extensions'

import FileEditor from './components/editor/FileEditor.vue'
import FileContextMenu from './components/FileContextMenu.vue'
import FileManagerError from './components/FileManagerError.vue'
import FileNavbar from './components/FileNavbar.vue'
import FileTableHeader from './components/FileTableHeader.vue'
import FileVirtualList from './components/FileVirtualList.vue'
import FileCreateItemModal from './components/modals/FileCreateItemModal.vue'
import FileDeleteItemModal from './components/modals/FileDeleteItemModal.vue'
import FileMoveItemModal from './components/modals/FileMoveItemModal.vue'
import FileRenameItemModal from './components/modals/FileRenameItemModal.vue'
import FileUploadConflictModal from './components/modals/FileUploadConflictModal.vue'
import FileUploadZipUrlModal from './components/modals/FileUploadZipUrlModal.vue'
import FileUploadDragAndDrop from './components/upload/FileUploadDragAndDrop.vue'
import { useFileSearch } from './composables/file-search'
import { useFileSelection } from './composables/file-selection'
import { useFileSorting } from './composables/file-sorting'
import { useFileUndoRedo } from './composables/file-undo-redo'
import { injectFileManager } from './providers/file-manager'
import type { FileItem } from './types'

defineProps<{
	showDebugInfo?: boolean
	showRefreshButton?: boolean
}>()

const { addNotification } = injectNotificationManager()
const ctx = injectFileManager()

const baseId = `files-${Math.random().toString(36).slice(2, 9)}`

const items = computed(() => ctx.items.value)
const isEditing = computed(() => ctx.editingFile.value !== null)
const isBusy = computed(() => ctx.isBusy?.value ?? false)
const busyTooltip = computed(() => ctx.busyTooltip?.value)
const activeOperations = computed(() => ctx.activeOperations?.value ?? [])

const uploadOverallProgress = computed(() => {
	const state = ctx.uploadState?.value
	if (!state || !state.isUploading || state.totalFiles === 0) return 0
	return Math.min((state.completedFiles + state.currentFileProgress) / state.totalFiles, 1)
})

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
	sortDesc: sortDescValue,
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

const { recordOperation, onKeydown } = useFileUndoRedo(
	(path, newName) => ctx.renameItem(path, newName),
	(source, dest) => ctx.moveItem(source, dest),
	() => ctx.refresh(),
	(title, text, type) => addNotification({ title, text, type }),
)

// Sticky observer for the table header
const fileUploadRef = ref<InstanceType<typeof FileUploadDragAndDrop>>()
const fileUploadEl = computed(() => fileUploadRef.value?.$el as HTMLElement | null)
const { isStuck: isLabelBarStuck } = useStickyObserver(fileUploadEl)

// Refs
const fileEditorRef = ref<InstanceType<typeof FileEditor>>()
const createItemModal = ref<InstanceType<typeof FileCreateItemModal>>()
const renameItemModal = ref<InstanceType<typeof FileRenameItemModal>>()
const moveItemModal = ref<InstanceType<typeof FileMoveItemModal>>()
const deleteItemModal = ref<InstanceType<typeof FileDeleteItemModal>>()
const uploadConflictModal = ref<InstanceType<typeof FileUploadConflictModal>>()
const uploadZipUrlModal = ref<InstanceType<typeof FileUploadZipUrlModal>>()
const contextMenuRef = ref<InstanceType<typeof FileContextMenu>>()

const newItemType = ref<'file' | 'directory'>('file')
const selectedItem = ref<FileItem | null>(null)

// Navigation
function navigateToSegment(index: number) {
	const newPath = index === -1 ? '/' : breadcrumbSegments.value.slice(0, index + 1).join('/')

	if (newPath === ctx.currentPath.value && !isEditing.value) {
		return
	}

	if (isEditing.value) {
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

function handleEditorClose() {
	ctx.stopEditing()
}

// CRUD handlers
async function handleCreateNewItem(name: string) {
	await ctx.createItem(name, newItemType.value)
}

async function handleRenameItem(newName: string) {
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
				title: 'Dry run failed',
				text: 'Error running dry run',
				type: 'error',
			})
		}
	} catch (error) {
		addNotification({
			title: 'Extract failed',
			text: error instanceof Error ? error.message : 'Unknown error',
			type: 'error',
		})
	}
}

async function handleExtractConfirm(path: string) {
	if (!ctx.extractFile) return
	try {
		await ctx.extractFile(path, true, false)
		addNotification({ title: 'Extraction started', type: 'success' })
	} catch (error) {
		addNotification({
			title: 'Extract failed',
			text: error instanceof Error ? error.message : 'Unknown error',
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
	if (isBusy.value) return
	selectedItem.value = item
	renameItemModal.value?.show(item)
}

function showMoveModal(item: FileItem) {
	if (isBusy.value) return
	selectedItem.value = item
	moveItemModal.value?.show()
}

function showDeleteModal(item: FileItem) {
	if (isBusy.value) return
	selectedItem.value = item
	deleteItemModal.value?.show()
}

function showBulkDeleteModal() {
	if (isBusy.value) return
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

function initiateFileUpload() {
	if (isBusy.value) return
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
	} else {
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

// Context menu
function handleContextMenu(item: FileItem, x: number, y: number) {
	const wd = isBusy.value
	const wdTooltip = busyTooltip.value
	const isZip = getFileExtension(item.name) === 'zip'

	const options = [
		{
			id: 'extract',
			shown: isZip && !!ctx.extractFile,
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => handleExtractItem(item),
		},
		{ divider: true, shown: isZip && !!ctx.extractFile },
		{
			id: 'rename',
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => showRenameModal(item),
		},
		{
			id: 'move',
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => showMoveModal(item),
		},
		{
			id: 'download',
			action: () => handleDownload(item),
			shown: item.type !== 'directory',
		},
		{
			id: 'delete',
			disabled: wd,
			tooltip: wd ? wdTooltip : undefined,
			action: () => showDeleteModal(item),
			color: 'red',
		},
	]

	contextMenuRef.value?.show(item, x, y, options)
}

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
