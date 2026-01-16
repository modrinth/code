<template>
	<FileCreateItemModal ref="createItemModal" :type="newItemType" @create="handleCreateNewItem" />
	<FileUploadConflictModal ref="uploadConflictModal" @proceed="extractItem" />
	<FileRenameItemModal ref="renameItemModal" :item="selectedItem" @rename="handleRenameItem" />
	<FileMoveItemModal
		ref="moveItemModal"
		:item="selectedItem"
		:current-path="currentPath"
		@move="handleMoveItem"
	/>
	<FileDeleteItemModal ref="deleteItemModal" :item="selectedItem" @delete="handleDeleteItem" />
	<Transition name="fade" mode="out-in">
		<div
			v-if="isLoading && items.length === 0"
			key="loading"
			class="mt-6 flex flex-col items-center justify-center gap-2 text-center text-secondary"
		>
			<SpinnerIcon class="animate-spin" />
			Loading files...
		</div>

		<div v-else key="content" class="contents">
			<div class="relative -mt-2 flex w-full flex-col">
				<div class="relative isolate flex w-full flex-col gap-2">
					<FileNavbar
						:breadcrumbs="breadcrumbSegments"
						:is-editing="isEditing"
						:editing-file-name="editingFile?.name"
						:editing-file-path="editingFile?.path"
						:is-editing-image="fileEditorRef?.isEditingImage"
						:search-query="searchQuery"
						:base-id="baseId"
						@navigate="navigateToSegment"
						@navigate-home="() => navigateToSegment(-1)"
						@prefetch-home="handlePrefetchHome"
						@update:search-query="searchQuery = $event"
						@create="showCreateModal"
						@upload="initiateFileUpload"
						@upload-zip="() => {}"
						@unzip-from-url="showUnzipFromUrlModal"
						@save="() => fileEditorRef?.saveFileContent(true)"
						@save-as="() => fileEditorRef?.saveFileContent(false)"
						@save-restart="() => fileEditorRef?.saveAndRestart()"
						@share="() => fileEditorRef?.shareToMclogs()"
					/>

					<div v-if="!isEditing" class="contents">
						<div ref="labelBarSentinel" class="h-0 w-full" aria-hidden="true" />
						<FileUploadDragAndDrop
							class="relative flex flex-col shadow-md"
							@files-dropped="handleDroppedFiles"
						>
							<FileLabelBar
								:sort-field="sortMethod"
								:sort-desc="sortDesc"
								:all-selected="allSelected"
								:some-selected="someSelected"
								:is-stuck="isLabelBarStuck"
								@sort="handleSort"
								@toggle-all="toggleSelectAll"
							/>
							<div
								v-for="op in ops"
								:key="`fs-op-${op.op}-${op.src}`"
								class="sticky top-20 z-20 grid grid-cols-[auto_1fr_auto] items-center gap-2 border-0 border-b-[1px] border-solid border-button-bg bg-table-alternateRow px-4 py-2 md:grid-cols-[auto_1fr_1fr_2fr_auto]"
							>
								<div>
									<PackageOpenIcon class="h-5 w-5 text-secondary" />
								</div>
								<div class="flex flex-wrap gap-x-4 gap-y-1 md:contents">
									<div
										class="flex items-center text-wrap break-all text-sm font-bold text-contrast"
									>
										Extracting
										{{ op.src.includes('https://') ? 'modpack from URL' : op.src }}
									</div>
									<span
										class="flex items-center gap-2 text-sm font-semibold"
										:class="{
											'text-green': op.state === 'done',
											'text-red': op.state?.startsWith('fail'),
											'text-orange': !op.state?.startsWith('fail') && op.state !== 'done',
										}"
									>
										<template v-if="op.state === 'done'">
											Done
											<CheckIcon style="stroke-width: 3px" />
										</template>
										<template v-else-if="op.state?.startsWith('fail')">
											Failed
											<XIcon style="stroke-width: 3px" />
										</template>
										<template v-else-if="op.state === 'cancelled'">
											<SpinnerIcon class="animate-spin" />
											Cancelling
										</template>
										<template v-else-if="op.state === 'queued'">
											<SpinnerIcon class="animate-spin" />
											Queued...
										</template>
										<template v-else-if="op.state === 'ongoing'">
											<SpinnerIcon class="animate-spin" />
											Extracting...
										</template>
										<template v-else>
											<UnknownIcon />
											Unknown state: {{ op.state }}
										</template>
									</span>
									<div class="col-span-2 flex grow flex-col gap-1 md:col-span-1 md:items-end">
										<div class="text-xs font-semibold text-contrast opacity-80">
											<span
												:class="{
													invisible: 'current_file' in op && !op.current_file,
												}"
											>
												{{
													'current_file' in op
														? (op.current_file?.split('/')?.pop() ?? 'unknown')
														: 'unknown'
												}}
											</span>
										</div>
										<ProgressBar
											:progress="'progress' in op ? op.progress : 0"
											:max="1"
											:color="
												op.state === 'done'
													? 'green'
													: op.state?.startsWith('fail')
														? 'red'
														: op.state === 'cancelled'
															? 'gray'
															: 'orange'
											"
											:waiting="op.state === 'queued' || !op.progress || op.progress === 0"
										/>
										<div
											class="text-xs text-secondary opacity-80"
											:class="{
												invisible: 'bytes_processed' in op && !op.bytes_processed,
											}"
										>
											{{ 'bytes_processed' in op ? formatBytes(op.bytes_processed) : '0 B' }}
											extracted
										</div>
									</div>
								</div>

								<div>
									<ButtonStyled circular>
										<button
											:disabled="!('id' in op) || !op.id"
											class="radial-progress-animation-overlay"
											:class="{ active: op.state === 'done' }"
											@click="
												() => {
													if ('id' in op && op.id) {
														dismissOrCancelOp(op.id, op.state === 'done' ? 'dismiss' : 'cancel')
													}
												}
											"
										>
											<XIcon />
										</button>
									</ButtonStyled>
								</div>
								<pre
									v-if="showDebugInfo"
									class="markdown-body col-span-full m-0 rounded-xl bg-button-bg text-xs"
									>{{ op }}</pre
								>
							</div>
							<FileUploadDropdown
								ref="uploadDropdownRef"
								class="border-0 border-t border-solid border-bg bg-table-alternateRow"
								:current-path="currentPath"
								@upload-complete="refreshList()"
							/>
							<Transition name="fade" mode="out-in">
								<div v-if="items.length > 0" key="list" class="h-full w-full overflow-hidden">
									<FileVirtualList
										:items="filteredItems"
										:selected-items="selectedItems"
										@extract="handleExtractItem"
										@delete="showDeleteModal"
										@rename="showRenameModal"
										@download="downloadFile"
										@move="showMoveModal"
										@move-direct-to="handleDirectMove"
										@edit="editFile"
										@hover="handleItemHover"
										@load-more="handleLoadMore"
										@toggle-select="toggleItemSelection"
									/>
								</div>
								<div
									v-else-if="items.length === 0 && !loadError"
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
									v-else-if="loadError"
									key="error"
									class="rounded-b-[20px]"
									title="Unable to load files"
									message="The folder may not exist."
									@refetch="refreshList"
									@home="navigateToSegment(-1)"
								/>
							</Transition>
						</FileUploadDragAndDrop>
					</div>
					<FileEditor
						v-else
						ref="fileEditorRef"
						:file="editingFile"
						:editor-component="VAceEditor"
						@close="handleEditorClose"
					/>
				</div>
			</div>

			<FloatingActionBar :shown="selectedItems.size > 0">
				<ButtonStyled circular>
					<button @click="deselectAll">
						<XIcon class="h-4 w-4" />
					</button>
				</ButtonStyled>
				<span class="text-sm font-medium text-contrast"> {{ selectedItems.size }} selected </span>
				<div class="ml-auto flex items-center gap-2">
					<ButtonStyled>
						<button @click="showBulkMoveModal">
							<RightArrowIcon class="h-4 w-4" />
							Move
						</button>
					</ButtonStyled>
					<ButtonStyled color="red">
						<button @click="showBulkDeleteModal">
							<TrashIcon class="h-4 w-4" />
							Delete
						</button>
					</ButtonStyled>
				</div>
			</FloatingActionBar>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import type { Archon, Kyros } from '@modrinth/api-client'
import {
	CheckIcon,
	FolderOpenIcon,
	PackageOpenIcon,
	RightArrowIcon,
	SpinnerIcon,
	TrashIcon,
	UnknownIcon,
	XIcon,
} from '@modrinth/assets'
import { formatBytes } from '@modrinth/utils'
import { useInfiniteQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, inject, onMounted, onUnmounted, provide, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ButtonStyled from '../../../components/base/ButtonStyled.vue'
import FloatingActionBar from '../../../components/base/FloatingActionBar.vue'
import ProgressBar from '../../../components/base/ProgressBar.vue'
import {
	FileEditor,
	FileLabelBar,
	FileManagerError,
	FileNavbar,
	FileUploadDragAndDrop,
	FileUploadDropdown,
	FileVirtualList,
} from '../../../components/servers/files'
import {
	FileCreateItemModal,
	FileDeleteItemModal,
	FileMoveItemModal,
	FileRenameItemModal,
	FileUploadConflictModal,
} from '../../../components/servers/files/modals'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '../../../providers'
import {
	getFileExtension,
	isEditableFile as isEditableFileCheck,
} from '../../../utils/file-extensions'

defineProps<{
	showDebugInfo?: boolean
}>()

const notifications = injectNotificationManager()
const { addNotification } = notifications
const client = injectModrinthClient()
const serverContext = injectModrinthServerContext()
const { serverId, fsOps, fsQueuedOps } = serverContext
const queryClient = useQueryClient()

interface BaseOperation {
	type: 'move' | 'rename'
	itemType: string
	fileName: string
}

interface MoveOperation extends BaseOperation {
	type: 'move'
	sourcePath: string
	destinationPath: string
}

interface RenameOperation extends BaseOperation {
	type: 'rename'
	path: string
	oldName: string
	newName: string
}

type Operation = MoveOperation | RenameOperation

interface InfiniteDirectoryData {
	pages: Kyros.Files.v0.DirectoryResponse[]
	pageParams: number[]
}

const modulesLoaded = inject<Promise<void>>('modulesLoaded')

const route = useRoute()
const router = useRouter()

const baseId = `files-${Math.random().toString(36).slice(2, 9)}`
const operationHistory = ref<Operation[]>([])
const redoStack = ref<Operation[]>([])

const searchQuery = ref('')
const sortMethod = ref('name')
const sortDesc = ref(false)

const selectedItems = ref<Set<string>>(new Set())

function toggleItemSelection(path: string) {
	const newSet = new Set(selectedItems.value)
	if (newSet.has(path)) {
		newSet.delete(path)
	} else {
		newSet.add(path)
	}
	selectedItems.value = newSet
}

function selectAll() {
	selectedItems.value = new Set(filteredItems.value.map((i) => i.path))
}

function deselectAll() {
	selectedItems.value = new Set()
}

function toggleSelectAll() {
	if (allSelected.value) {
		deselectAll()
	} else {
		selectAll()
	}
}

const allSelected = computed(
	() => filteredItems.value.length > 0 && selectedItems.value.size === filteredItems.value.length,
)

const someSelected = computed(
	() => selectedItems.value.size > 0 && selectedItems.value.size < filteredItems.value.length,
)

const currentPath = computed(() => (typeof route.query.path === 'string' ? route.query.path : '/'))

const createItemModal = ref<InstanceType<typeof FileCreateItemModal>>()
const renameItemModal = ref<InstanceType<typeof FileRenameItemModal>>()
const moveItemModal = ref<InstanceType<typeof FileMoveItemModal>>()
const deleteItemModal = ref<InstanceType<typeof FileDeleteItemModal>>()
const uploadConflictModal = ref<InstanceType<typeof FileUploadConflictModal>>()

const newItemType = ref<'file' | 'directory'>('file')
const selectedItem = ref<Kyros.Files.v0.DirectoryItem | null>(null)

const isEditing = ref(false)
const editingFile = ref<{ name: string; type: string; path: string } | null>(null)
const fileEditorRef = ref<InstanceType<typeof FileEditor>>()

const uploadDropdownRef = ref<InstanceType<typeof FileUploadDropdown>>()

const VAceEditor = ref()

const labelBarSentinel = ref<HTMLDivElement>()
const isLabelBarStuck = ref(false)
let labelBarObserver: IntersectionObserver | null = null

const viewFilter = ref('all')

const {
	data: directoryData,
	isLoading,
	error: loadError,
	fetchNextPage,
	hasNextPage,
	isFetchingNextPage,
} = useInfiniteQuery({
	queryKey: computed(() => ['files', serverId, currentPath.value]),
	queryFn: async ({ pageParam = 1 }) => {
		if (modulesLoaded) await modulesLoaded
		return client.kyros.files_v0.listDirectory(currentPath.value, pageParam, 100)
	},
	getNextPageParam: (lastPage, allPages) => {
		const pageSize = 100
		if (lastPage.items.length >= pageSize) {
			return allPages.length + 1
		}

		if (lastPage.current < lastPage.total) {
			return lastPage.current + 1
		}
		return undefined
	},
	staleTime: 30_000,
	initialPageParam: 1,
})

const items = computed(() => directoryData.value?.pages.flatMap((page) => page.items) ?? [])

function prefetchDirectory(path: string) {
	queryClient.prefetchInfiniteQuery({
		queryKey: ['files', serverId, path],
		queryFn: async () => {
			if (modulesLoaded) await modulesLoaded
			try {
				return await client.kyros.files_v0.listDirectory(path, 1, 100)
			} catch {
				return { items: [], total: 0, current: 1 }
			}
		},
		initialPageParam: 1,
		staleTime: 30_000,
	})
}

let prefetchTimeout: ReturnType<typeof setTimeout> | null = null
let prefetchHomeTimeout: ReturnType<typeof setTimeout> | null = null

function handlePrefetchHome() {
	if (prefetchHomeTimeout) {
		clearTimeout(prefetchHomeTimeout)
		prefetchHomeTimeout = null
	}

	prefetchHomeTimeout = setTimeout(() => {
		prefetchDirectory('/')
	}, 150)
}

function prefetchFileContent(path: string) {
	queryClient.prefetchQuery({
		queryKey: ['file-content', serverId, path],
		queryFn: async () => {
			if (modulesLoaded) await modulesLoaded
			try {
				const blob = await client.kyros.files_v0.downloadFile(path)
				return await blob.text()
			} catch {
				return null
			}
		},
		staleTime: 30_000,
	})
}

function handleItemHover(item: { type: string; path: string; name: string }) {
	if (prefetchTimeout) {
		clearTimeout(prefetchTimeout)
		prefetchTimeout = null
	}

	if (item.type === 'directory') {
		prefetchTimeout = setTimeout(() => {
			const routePath = typeof route.query.path === 'string' ? route.query.path : ''
			const navPath = routePath.endsWith('/')
				? `${routePath}${item.name}`
				: `${routePath}/${item.name}`
			prefetchDirectory(navPath)
		}, 150)
	} else if (item.type === 'file') {
		const ext = getFileExtension(item.name)
		if (isEditableFileCheck(ext)) {
			prefetchTimeout = setTimeout(() => {
				prefetchFileContent(item.path)
			}, 150)
		}
	}
}

function getQueryKey() {
	return ['files', serverId, currentPath.value]
}

const deleteMutation = useMutation({
	mutationFn: ({ path, recursive }: { path: string; recursive: boolean }) =>
		client.kyros.files_v0.deleteFileOrFolder(path, recursive),

	onMutate: async ({ path }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)

		queryClient.setQueryData(queryKey, (old: InfiniteDirectoryData | undefined) => {
			if (!old) return old
			return {
				...old,
				pages: old.pages.map((page) => ({
					...page,
					items: page.items.filter((item) => item.path !== path),
					total: Math.max(0, page.total - 1),
				})),
			}
		})
		return { previous }
	},

	onError: (err: Error, _vars, context) => {
		queryClient.setQueryData(getQueryKey(), context?.previous)
		addNotification({ title: 'Delete failed', text: err.message, type: 'error' })
	},

	onSuccess: () => {
		addNotification({ title: 'File deleted', text: 'Your file has been deleted.', type: 'success' })
	},

	onSettled: () => {
		queryClient.invalidateQueries({ queryKey: ['files', serverId] })
	},
})

const renameMutation = useMutation({
	mutationFn: ({ path, newName }: { path: string; newName: string }) =>
		client.kyros.files_v0.renameFileOrFolder(path, newName),

	onMutate: async ({ path, newName }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)

		queryClient.setQueryData(queryKey, (old: InfiniteDirectoryData | undefined) => {
			if (!old) return old
			return {
				...old,
				pages: old.pages.map((page) => ({
					...page,
					items: page.items.map((item) =>
						item.path === path
							? { ...item, name: newName, path: item.path.replace(/[^/]+$/, newName) }
							: item,
					),
				})),
			}
		})
		return { previous }
	},

	onError: (err: Error, _vars, context) => {
		queryClient.setQueryData(getQueryKey(), context?.previous)
		addNotification({ title: 'Rename failed', text: err.message, type: 'error' })
	},

	onSuccess: (_, { newName }) => {
		addNotification({ title: 'Renamed', text: `Renamed to ${newName}`, type: 'success' })
	},

	onSettled: () => {
		queryClient.invalidateQueries({ queryKey: ['files', serverId] })
	},
})

const moveMutation = useMutation({
	mutationFn: ({ source, destination }: { source: string; destination: string }) =>
		client.kyros.files_v0.moveFileOrFolder(source, destination),

	onMutate: async ({ source }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)

		queryClient.setQueryData(queryKey, (old: InfiniteDirectoryData | undefined) => {
			if (!old) return old
			return {
				...old,
				pages: old.pages.map((page) => ({
					...page,
					items: page.items.filter((item) => item.path !== source),
					total: Math.max(0, page.total - 1),
				})),
			}
		})
		return { previous }
	},

	onError: (err: Error, _vars, context) => {
		queryClient.setQueryData(getQueryKey(), context?.previous)
		addNotification({ title: 'Move failed', text: err.message, type: 'error' })
	},

	onSuccess: (_, { destination }) => {
		addNotification({ title: 'Moved', text: `Moved to ${destination}`, type: 'success' })
	},

	onSettled: () => {
		queryClient.invalidateQueries({ queryKey: ['files', serverId] })
	},
})

const createMutation = useMutation({
	mutationFn: ({ path, type }: { path: string; type: 'file' | 'directory' }) =>
		client.kyros.files_v0.createFileOrFolder(path, type),

	onMutate: async ({ path, type }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)

		const name = path.split('/').pop()!
		const now = Math.floor(Date.now() / 1000)

		const newItem: Kyros.Files.v0.DirectoryItem = {
			name,
			path,
			type,
			modified: now,
			created: now,
			...(type === 'directory' ? { count: 0 } : { size: 0 }),
		}
		queryClient.setQueryData(queryKey, (old: InfiniteDirectoryData | undefined) => {
			if (!old) return old
			return {
				...old,
				pages: old.pages.map((page, i) =>
					i === 0
						? {
								...page,
								items: [newItem, ...page.items],
								total: page.total + 1,
							}
						: page,
				),
			}
		})
		return { previous }
	},

	onError: (err: Error, _vars, context) => {
		queryClient.setQueryData(getQueryKey(), context?.previous)
		addNotification({ title: 'Create failed', text: err.message, type: 'error' })
	},

	onSuccess: (_, { path, type }) => {
		const name = path.split('/').pop()
		addNotification({
			title: `${type === 'directory' ? 'Folder' : 'File'} created`,
			text: `Created ${name}`,
			type: 'success',
		})
	},

	onSettled: () => {
		queryClient.invalidateQueries({ queryKey: ['files', serverId] })
	},
})

const extractMutation = useMutation({
	mutationFn: ({ path, override }: { path: string; override: boolean }) =>
		client.kyros.files_v0.extractFile(path, override, false),

	onSuccess: () => {
		addNotification({ title: 'Extraction started', type: 'success' })
	},

	onError: (err: Error) => {
		addNotification({ title: 'Extract failed', text: err.message, type: 'error' })
	},

	onSettled: () => {
		queryClient.invalidateQueries({ queryKey: ['files', serverId] })
	},
})

function refreshList() {
	queryClient.invalidateQueries({ queryKey: ['files', serverId] })
}

async function undoLastOperation() {
	const lastOperation = operationHistory.value.pop()
	if (!lastOperation) return

	try {
		switch (lastOperation.type) {
			case 'move':
				await client.kyros.files_v0.moveFileOrFolder(
					`${lastOperation.destinationPath}/${lastOperation.fileName}`.replace('//', '/'),
					`${lastOperation.sourcePath}/${lastOperation.fileName}`.replace('//', '/'),
				)
				break
			case 'rename':
				await client.kyros.files_v0.renameFileOrFolder(
					`${lastOperation.path}/${lastOperation.newName}`.replace('//', '/'),
					lastOperation.oldName,
				)
				break
		}

		redoStack.value.push(lastOperation)

		refreshList()
		addNotification({
			title: `${lastOperation.type === 'move' ? 'Move' : 'Rename'} undone`,
			text: `${lastOperation.fileName} has been restored to its original ${lastOperation.type === 'move' ? 'location' : 'name'}`,
			type: 'success',
		})
	} catch (error) {
		console.error(`Error undoing ${lastOperation.type}:`, error)
		addNotification({
			title: 'Undo failed',
			text: `Failed to undo the last ${lastOperation.type} operation`,
			type: 'error',
		})
	}
}

async function redoLastOperation() {
	const lastOperation = redoStack.value.pop()
	if (!lastOperation) return

	try {
		switch (lastOperation.type) {
			case 'move':
				await client.kyros.files_v0.moveFileOrFolder(
					`${lastOperation.sourcePath}/${lastOperation.fileName}`.replace('//', '/'),
					`${lastOperation.destinationPath}/${lastOperation.fileName}`.replace('//', '/'),
				)
				break
			case 'rename':
				await client.kyros.files_v0.renameFileOrFolder(
					`${lastOperation.path}/${lastOperation.oldName}`.replace('//', '/'),
					lastOperation.newName,
				)
				break
		}

		operationHistory.value.push(lastOperation)

		refreshList()
		addNotification({
			title: `${lastOperation.type === 'move' ? 'Move' : 'Rename'} redone`,
			text: `${lastOperation.fileName} has been ${lastOperation.type === 'move' ? 'moved' : 'renamed'} again`,
			type: 'success',
		})
	} catch (error) {
		console.error(`Error redoing ${lastOperation.type}:`, error)
		addNotification({
			title: 'Redo failed',
			text: `Failed to redo the last ${lastOperation.type} operation`,
			type: 'error',
		})
	}
}

function handleCreateNewItem(name: string) {
	const path = `${currentPath.value}/${name}`.replace('//', '/')
	createMutation.mutate({ path, type: newItemType.value })
}

function handleRenameItem(newName: string) {
	const item = selectedItem.value
	if (!item) return

	const path = `${currentPath.value}/${item.name}`.replace('//', '/')

	renameMutation.mutate(
		{ path, newName },
		{
			onSuccess: () => {
				redoStack.value = []
				operationHistory.value.push({
					type: 'rename',
					itemType: item.type,
					fileName: item.name,
					path: currentPath.value,
					oldName: item.name,
					newName,
				})
			},
		},
	)
}

const localQueuedOps = ref<Archon.Websocket.v0.QueuedFilesystemOp[]>([])

function extractItem(path: string) {
	localQueuedOps.value.push({ op: 'unarchive', src: path })
	setTimeout(() => {
		localQueuedOps.value = localQueuedOps.value.filter(
			(x) => x.op !== 'unarchive' || x.src !== path,
		)
	}, 4000)

	extractMutation.mutate(
		{ path, override: true },
		{
			onError: () => {
				localQueuedOps.value = localQueuedOps.value.filter(
					(x) => x.op !== 'unarchive' || x.src !== path,
				)
			},
		},
	)
}

async function handleExtractItem(item: { name: string; type: string; path: string }) {
	try {
		const dry = await client.kyros.files_v0.extractFile(item.path, true, true)
		if (dry) {
			if (dry.conflicting_files.length === 0) {
				extractItem(item.path)
			} else {
				uploadConflictModal.value?.show(item.path, dry.conflicting_files)
			}
		} else {
			addNotification({ title: 'Dry run failed', text: 'Error running dry run', type: 'error' })
		}
	} catch (error) {
		console.error('Error extracting item:', error)
		addNotification({
			title: 'Extract failed',
			text: error instanceof Error ? error.message : 'Unknown error',
			type: 'error',
		})
	}
}

function handleMoveItem(destination: string) {
	const item = selectedItem.value
	if (!item) return

	const sourcePath = currentPath.value
	const source = `${sourcePath}/${item.name}`.replace('//', '/')
	const dest = `${destination}/${item.name}`.replace('//', '/')

	moveMutation.mutate(
		{ source, destination: dest },
		{
			onSuccess: () => {
				redoStack.value = []
				operationHistory.value.push({
					type: 'move',
					sourcePath,
					destinationPath: destination,
					fileName: item.name,
					itemType: item.type,
				})
			},
		},
	)
}

function handleDirectMove(moveData: {
	name: string
	type: string
	path: string
	destination: string
}) {
	const dest = `${moveData.destination}/${moveData.name}`.replace('//', '/')
	const sourcePath = moveData.path.substring(0, moveData.path.lastIndexOf('/'))

	moveMutation.mutate(
		{ source: moveData.path, destination: dest },
		{
			onSuccess: () => {
				redoStack.value = []
				operationHistory.value.push({
					type: 'move',
					sourcePath,
					destinationPath: moveData.destination,
					fileName: moveData.name,
					itemType: moveData.type,
				})
			},
		},
	)
}

function handleDeleteItem() {
	const item = selectedItem.value
	if (!item) return

	const path = `${currentPath.value}/${item.name}`.replace('//', '/')
	deleteMutation.mutate({ path, recursive: item.type === 'directory' })
}

function showCreateModal(type: 'file' | 'directory') {
	newItemType.value = type
	createItemModal.value?.show()
}

function showUnzipFromUrlModal(_cf: boolean) {
	// TODO: Implement unzip from URL modal
	addNotification({
		title: 'Not implemented',
		text: 'Unzip from URL is not yet implemented',
		type: 'info',
	})
}

function showRenameModal(item: Kyros.Files.v0.DirectoryItem) {
	selectedItem.value = item
	renameItemModal.value?.show(item)
}

function showMoveModal(item: Kyros.Files.v0.DirectoryItem) {
	selectedItem.value = item
	moveItemModal.value?.show()
}

function showDeleteModal(item: Kyros.Files.v0.DirectoryItem) {
	selectedItem.value = item
	deleteItemModal.value?.show()
}

async function showBulkMoveModal() {
	addNotification({
		title: 'Bulk move',
		text: `Moving ${selectedItems.value.size} items is not yet implemented`,
		type: 'info',
	})
}

async function showBulkDeleteModal() {
	if (selectedItems.value.size === 0) return

	const itemsToDelete = Array.from(selectedItems.value)

	for (const path of itemsToDelete) {
		const item = items.value.find((i) => i.path === path)
		if (item) {
			deleteMutation.mutate({ path, recursive: item.type === 'directory' })
		}
	}

	deselectAll()
}

function handleSort(field: string) {
	if (sortMethod.value === field) {
		sortDesc.value = !sortDesc.value
	} else {
		sortMethod.value = field
		sortDesc.value = false
	}
}

function applySort(items: Kyros.Files.v0.DirectoryItem[]) {
	let result = [...items]

	switch (viewFilter.value) {
		case 'filesOnly':
			result = result.filter((item) => item.type !== 'directory')
			break
		case 'foldersOnly':
			result = result.filter((item) => item.type === 'directory')
			break
	}

	function compareItems(a: Kyros.Files.v0.DirectoryItem, b: Kyros.Files.v0.DirectoryItem) {
		if (viewFilter.value === 'all') {
			if (a.type === 'directory' && b.type !== 'directory') return -1
			if (a.type !== 'directory' && b.type === 'directory') return 1
		}

		switch (sortMethod.value) {
			case 'modified':
				return sortDesc.value ? a.modified - b.modified : b.modified - a.modified
			case 'created':
				return sortDesc.value ? a.created - b.created : b.created - a.created
			case 'size': {
				const aValue =
					a.type === 'directory'
						? 'count' in a && a.count !== undefined
							? a.count
							: 0
						: 'size' in a && a.size !== undefined
							? a.size
							: 0
				const bValue =
					b.type === 'directory'
						? 'count' in b && b.count !== undefined
							? b.count
							: 0
						: 'size' in b && b.size !== undefined
							? b.size
							: 0
				return sortDesc.value ? aValue - bValue : bValue - aValue
			}
			default:
				return sortDesc.value ? b.name.localeCompare(a.name) : a.name.localeCompare(b.name)
		}
	}

	result.sort(compareItems)
	return result
}

const filteredItems = computed(() => {
	let result = [...items.value]

	if (searchQuery.value) {
		const query = searchQuery.value.toLowerCase()
		result = result.filter((item) => item.name.toLowerCase().includes(query))
	}

	return applySort(result)
})

async function handleLoadMore() {
	if (isFetchingNextPage.value || !hasNextPage.value) return
	await fetchNextPage()
}

function onKeydown(e: KeyboardEvent) {
	if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'z') {
		e.preventDefault()
		undoLastOperation()
	}
	if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'z') {
		e.preventDefault()
		redoLastOperation()
	}
}

function editFile(item: { name: string; type: string; path: string }) {
	editingFile.value = item
	isEditing.value = true
	router.push({ query: { ...route.query, path: currentPath.value, editing: item.path } })
}

function initializeFileEdit() {
	if (!route.query.editing) return

	const filePath = route.query.editing as string
	editingFile.value = {
		name: filePath.split('/').pop() || '',
		type: 'file',
		path: filePath,
	}
	isEditing.value = true
}

function handleEditorClose() {
	isEditing.value = false
	editingFile.value = null
	const newQuery = { ...route.query }
	delete newQuery.editing
	router.replace({ query: newQuery })
}

onMounted(async () => {
	if (modulesLoaded) await modulesLoaded

	if (typeof window !== 'undefined') {
		const { VAceEditor: Ace } = await import('vue3-ace-editor')
		await Promise.all([
			import('ace-builds/src-noconflict/mode-json'),
			import('ace-builds/src-noconflict/mode-yaml'),
			import('ace-builds/src-noconflict/mode-toml'),
			import('ace-builds/src-noconflict/mode-sh'),
			import('ace-builds/src-noconflict/mode-batchfile'),
			import('ace-builds/src-noconflict/mode-powershell'),
			import('ace-builds/src-noconflict/mode-java'),
			import('ace-builds/src-noconflict/mode-javascript'),
			import('ace-builds/src-noconflict/mode-typescript'),
			import('ace-builds/src-noconflict/mode-python'),
			import('ace-builds/src-noconflict/mode-ruby'),
			import('ace-builds/src-noconflict/mode-php'),
			import('ace-builds/src-noconflict/mode-html'),
			import('ace-builds/src-noconflict/mode-css'),
			import('ace-builds/src-noconflict/mode-c_cpp'),
			import('ace-builds/src-noconflict/mode-rust'),
			import('ace-builds/src-noconflict/mode-golang'),
			import('ace-builds/src-noconflict/mode-markdown'),
			import('ace-builds/src-noconflict/mode-properties'),
			import('ace-builds/src-noconflict/mode-ini'),
			import('ace-builds/src-noconflict/mode-text'),
			import('../../../utils/ace-theme.ts'),
		])
		VAceEditor.value = Ace
	}

	initializeFileEdit()

	document.addEventListener('keydown', onKeydown)

	localQueuedOps.value = []
})

onUnmounted(() => {
	document.removeEventListener('keydown', onKeydown)
	labelBarObserver?.disconnect()
})

type QueuedOpWithState = Archon.Websocket.v0.QueuedFilesystemOp & { state: 'queued' }

const ops = computed<(QueuedOpWithState | Archon.Websocket.v0.FilesystemOperation)[]>(() => [
	...localQueuedOps.value.map((x) => ({ ...x, state: 'queued' }) satisfies QueuedOpWithState),
	...fsQueuedOps.value.map((x) => ({ ...x, state: 'queued' }) satisfies QueuedOpWithState),
	...fsOps.value,
])

async function dismissOrCancelOp(opId: string, action: 'dismiss' | 'cancel') {
	try {
		await client.kyros.files_v0.modifyOperation(opId, action)
	} catch (error) {
		console.error(`Failed to ${action} operation:`, error)
	}
}

watch(
	() => fsOps.value,
	() => {
		refreshList()
	},
)

watch(
	labelBarSentinel,
	(newSentinel) => {
		// Disconnect any existing observer
		if (labelBarObserver) {
			labelBarObserver.disconnect()
			labelBarObserver = null
		}

		// Create new observer when sentinel becomes available
		if (newSentinel) {
			labelBarObserver = new IntersectionObserver(
				([entry]) => {
					isLabelBarStuck.value = !entry.isIntersecting
				},
				{ threshold: 0 },
			)
			labelBarObserver.observe(newSentinel)
		}
	},
	{ flush: 'post' },
)

watch(
	() => route.query,
	(newQuery, oldQuery) => {
		if (newQuery.path !== oldQuery?.path) {
			searchQuery.value = ''
			viewFilter.value = 'all'
			sortMethod.value = 'name'
			sortDesc.value = false
			deselectAll()
		}

		if (newQuery.editing && editingFile.value?.path !== newQuery.editing) {
			editingFile.value = {
				name: (newQuery.editing as string).split('/').pop() || '',
				type: 'file',
				path: newQuery.editing as string,
			}
			isEditing.value = true
		} else if (oldQuery?.editing && !newQuery.editing) {
			isEditing.value = false
			editingFile.value = null
		}
	},
	{ deep: true },
)

const breadcrumbSegments = computed(() => {
	if (typeof currentPath.value === 'string') {
		return currentPath.value.split('/').filter(Boolean)
	}
	return []
})

function navigateToSegment(index: number) {
	const newPath = index === -1 ? '/' : breadcrumbSegments.value.slice(0, index + 1).join('/')

	// Don't navigate if already at target path (unless editing, to close editor)
	if (newPath === currentPath.value && !isEditing.value) {
		return
	}

	router.push({ query: { ...route.query, path: newPath } })
	if (isEditing.value) {
		isEditing.value = false
		editingFile.value = null

		const newQuery = { ...route.query }
		delete newQuery.editing
		router.replace({ query: newQuery })
	}
}

function handleDroppedFiles(files: File[]) {
	if (isEditing.value) return

	files.forEach((file) => {
		uploadDropdownRef.value?.uploadFile(file)
	})
}

function initiateFileUpload() {
	const input = document.createElement('input')
	input.type = 'file'
	input.multiple = true
	input.onchange = () => {
		if (input.files) {
			Array.from(input.files).forEach((file) => {
				uploadDropdownRef.value?.uploadFile(file)
			})
		}
	}
	input.click()
}

async function downloadFile(item: Kyros.Files.v0.DirectoryItem) {
	if (item.type === 'file') {
		try {
			const path = `${currentPath.value}/${item.name}`.replace('//', '/')
			const fileData = await client.kyros.files_v0.downloadFile(path)
			if (fileData) {
				const blob = new Blob([fileData], { type: 'application/octet-stream' })
				const link = document.createElement('a')
				link.href = window.URL.createObjectURL(blob)
				link.download = item.name
				link.click()
				window.URL.revokeObjectURL(link.href)
			} else {
				throw new Error('File data is undefined')
			}
		} catch (error) {
			console.error('Error downloading file:', error)
			addNotification({
				title: 'Download failed',
				text: 'Could not download the file.',
				type: 'error',
			})
		}
	}
}

provide('modulesLoaded', modulesLoaded)
</script>

<style scoped>
.radial-progress-animation-overlay {
	position: relative;
}

@property --_radial-percentage {
	syntax: '<percentage>';
	inherits: false;
	initial-value: 0%;
}

.radial-progress-animation-overlay.active::before {
	animation: radial-progress 3s linear forwards;
}

.radial-progress-animation-overlay::before {
	content: '';
	inset: -2px;
	position: absolute;
	border-radius: 50%;
	box-sizing: content-box;
	border: 2px solid var(--color-button-bg);
	filter: brightness(var(--hover-brightness));
	mask-image: conic-gradient(
		black 0%,
		black var(--_radial-percentage),
		transparent var(--_radial-percentage),
		transparent 100%
	);
}

@keyframes radial-progress {
	from {
		--_radial-percentage: 0%;
	}
	to {
		--_radial-percentage: 100%;
	}
}

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
