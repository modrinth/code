<template>
	<div data-pyro-file-manager-root class="contents">
		<FilesCreateItemModal ref="createItemModal" :type="newItemType" @create="handleCreateNewItem" />
		<FilesUploadZipUrlModal ref="uploadZipModal" />
		<FilesUploadConflictModal ref="uploadConflictModal" @proceed="extractItem" />

		<FilesRenameItemModal ref="renameItemModal" :item="selectedItem" @rename="handleRenameItem" />

		<FilesMoveItemModal
			ref="moveItemModal"
			:item="selectedItem"
			:current-path="currentPath"
			@move="handleMoveItem"
		/>

		<FilesDeleteItemModal ref="deleteItemModal" :item="selectedItem" @delete="handleDeleteItem" />

		<FilesUploadDragAndDrop
			class="relative flex w-full flex-col rounded-2xl border border-solid border-bg-raised"
			@files-dropped="handleDroppedFiles"
		>
			<div ref="mainContent" class="relative isolate flex w-full flex-col">
				<div v-if="!isEditing" class="contents">
					<FilesBrowseNavbar
						:breadcrumb-segments="breadcrumbSegments"
						:search-query="searchQuery"
						:current-filter="viewFilter"
						:base-id="`browse-navbar-${baseId}`"
						@navigate="navigateToSegment"
						@create="showCreateModal"
						@upload="initiateFileUpload"
						@upload-zip="() => {}"
						@unzip-from-url="showUnzipFromUrlModal"
						@filter="handleFilter"
						@update:search-query="searchQuery = $event"
					/>
					<FilesLabelBar :sort-field="sortMethod" :sort-desc="sortDesc" @sort="handleSort" />
					<div
						v-for="op in ops"
						:key="`fs-op-${op.op}-${op.src}`"
						class="sticky top-20 z-20 grid grid-cols-[auto_1fr_auto] items-center gap-2 border-0 border-b-[1px] border-solid border-button-bg bg-table-alternateRow px-4 py-2 md:grid-cols-[auto_1fr_1fr_2fr_auto]"
					>
						<div>
							<PackageOpenIcon class="h-5 w-5 text-secondary" />
						</div>
						<div class="flex flex-wrap gap-x-4 gap-y-1 md:contents">
							<div class="flex items-center text-wrap break-all text-sm font-bold text-contrast">
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
							v-if="flags.advancedDebugInfo"
							class="markdown-body col-span-full m-0 rounded-xl bg-button-bg text-xs"
							>{{ op }}</pre
						>
					</div>
					<FilesUploadDropdown
						ref="uploadDropdownRef"
						class="rounded-b-xl border-0 border-t border-solid border-bg bg-table-alternateRow"
						:current-path="currentPath"
						@upload-complete="refreshList()"
					/>
				</div>
				<FilesEditingNavbar
					v-else
					:file-name="editingFile?.name"
					:is-image="isEditingImage"
					:file-path="editingFile?.path"
					:breadcrumb-segments="breadcrumbSegments"
					@cancel="cancelEditing"
					@save="() => saveFileContent(true)"
					@save-as="saveFileContentAs"
					@save-restart="saveFileContentRestart"
					@share="requestShareLink"
					@navigate="navigateToSegment"
				/>

				<div v-if="isEditing" class="h-full w-full flex-grow">
					<component
						:is="VAceEditor"
						v-if="!isEditingImage"
						v-model:value="fileContent"
						:lang="
							(() => {
								const ext = editingFile?.name?.split('.')?.pop()?.toLowerCase() ?? ''
								return ext === 'json'
									? 'json'
									: ext === 'toml'
										? 'toml'
										: ext === 'sh'
											? 'sh'
											: ['yml', 'yaml'].includes(ext)
												? 'yaml'
												: 'text'
							})()
						"
						theme="one_dark"
						:print-margin="false"
						style="height: 750px; font-size: 1rem"
						class="ace_editor ace_hidpi ace-one-dark ace_dark rounded-b-lg"
						@init="onInit"
					/>
					<FilesImageViewer v-else :image-blob="imagePreview" />
				</div>
				<div v-else-if="items.length > 0" class="h-full w-full overflow-hidden rounded-b-2xl">
					<FileVirtualList
						:items="filteredItems"
						@extract="handleExtractItem"
						@delete="showDeleteModal"
						@rename="showRenameModal"
						@download="downloadFile"
						@move="showMoveModal"
						@move-direct-to="handleDirectMove"
						@edit="editFile"
						@hover="handleItemHover"
						@contextmenu="showContextMenu"
						@load-more="handleLoadMore"
					/>
				</div>

				<div
					v-else-if="!isLoading && items.length === 0 && !loadError"
					class="flex h-full w-full items-center justify-center p-20"
				>
					<div class="flex flex-col items-center gap-4 text-center">
						<FolderOpenIcon class="h-16 w-16 text-secondary" />
						<h3 class="m-0 text-2xl font-bold text-contrast">This folder is empty</h3>
						<p class="m-0 text-sm text-secondary">There are no files or folders.</p>
					</div>
				</div>

				<FileManagerError
					v-else-if="loadError"
					title="Unable to load files"
					message="The folder may not exist."
					@refetch="refreshList"
					@home="navigateToSegment(-1)"
				/>
			</div>

			<div
				v-if="isDragging"
				class="absolute inset-0 flex items-center justify-center rounded-2xl bg-black bg-opacity-50 text-white"
			>
				<div class="text-center">
					<UploadIcon class="mx-auto h-16 w-16" />
					<p class="mt-2 text-xl">Drop files here to upload</p>
				</div>
			</div>
		</FilesUploadDragAndDrop>

		<FilesContextMenu
			ref="contextMenu"
			:item="contextMenuInfo.item"
			:x="contextMenuInfo.x"
			:y="contextMenuInfo.y"
			:is-at-bottom="isAtBottom"
			@rename="showRenameModal"
			@move="showMoveModal"
			@download="downloadFile"
			@delete="showDeleteModal"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Kyros } from '@modrinth/api-client'
import {
	CheckIcon,
	FolderOpenIcon,
	PackageOpenIcon,
	SpinnerIcon,
	UnknownIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	ProgressBar,
} from '@modrinth/ui'
import type { FilesystemOp, FSQueuedOp } from '@modrinth/utils'
import { formatBytes } from '@modrinth/utils'
import { useInfiniteQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

import FileManagerError from '~/components/ui/servers/FileManagerError.vue'
import FilesBrowseNavbar from '~/components/ui/servers/FilesBrowseNavbar.vue'
import FilesContextMenu from '~/components/ui/servers/FilesContextMenu.vue'
import FilesCreateItemModal from '~/components/ui/servers/FilesCreateItemModal.vue'
import FilesDeleteItemModal from '~/components/ui/servers/FilesDeleteItemModal.vue'
import FilesEditingNavbar from '~/components/ui/servers/FilesEditingNavbar.vue'
import FilesImageViewer from '~/components/ui/servers/FilesImageViewer.vue'
import FilesLabelBar from '~/components/ui/servers/FilesLabelBar.vue'
import FilesMoveItemModal from '~/components/ui/servers/FilesMoveItemModal.vue'
import FilesRenameItemModal from '~/components/ui/servers/FilesRenameItemModal.vue'
import FilesUploadConflictModal from '~/components/ui/servers/FilesUploadConflictModal.vue'
import FilesUploadDragAndDrop from '~/components/ui/servers/FilesUploadDragAndDrop.vue'
import FilesUploadDropdown from '~/components/ui/servers/FilesUploadDropdown.vue'
import FilesUploadZipUrlModal from '~/components/ui/servers/FilesUploadZipUrlModal.vue'
import FileVirtualList from '~/components/ui/servers/FileVirtualList.vue'
import type { ModrinthServer } from '~/composables/servers/modrinth-servers.ts'
import { handleServersError } from '~/composables/servers/modrinth-servers.ts'

const notifications = injectNotificationManager()
const { addNotification } = notifications
const client = injectModrinthClient()
const serverContext = injectModrinthServerContext()
const { fsOps, fsQueuedOps } = serverContext
const flags = useFeatureFlags()
const baseId = useId()
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

const props = defineProps<{
	server: ModrinthServer
}>()

const modulesLoaded = inject<Promise<void>>('modulesLoaded')

const route = useRoute()
const router = useRouter()

const VAceEditor = ref()
const mainContent = ref<HTMLElement | null>(null)
const contextMenu = ref()
const operationHistory = ref<Operation[]>([])
const redoStack = ref<Operation[]>([])

const searchQuery = ref('')
const sortMethod = ref('name')
const sortDesc = ref(false)

const serverId = computed(() => props.server.serverId)
const currentPath = computed(() => (typeof route.query.path === 'string' ? route.query.path : '/'))

const isAtBottom = ref(false)
const contextMenuInfo = ref<any>({ item: null, x: 0, y: 0 })

const createItemModal = ref()
const renameItemModal = ref()
const moveItemModal = ref()
const deleteItemModal = ref()
const uploadZipModal = ref()
const uploadConflictModal = ref()

const newItemType = ref<'file' | 'directory'>('file')
const selectedItem = ref<any>(null)
const fileContent = ref('')

const isEditing = ref(false)
const editingFile = ref<any>(null)
const closeEditor = ref(false)
const isEditingImage = ref(false)
const imagePreview = ref()

const isDragging = ref(false)

const uploadDropdownRef = ref()

const data = computed(() => props.server.general)

const viewFilter = ref('all')

function handleFilter(type: string) {
	viewFilter.value = type
	sortMethod.value = 'name'
	sortDesc.value = false
}

useHead({
	title: computed(() => `Files - ${data.value?.name ?? 'Server'} - Modrinth`),
})

const {
	data: directoryData,
	isLoading,
	error: loadError,
	fetchNextPage,
	hasNextPage,
	isFetchingNextPage,
} = useInfiniteQuery({
	queryKey: computed(() => ['files', serverId.value, currentPath.value]),
	queryFn: async ({ pageParam = 1 }) => {
		console.log('[query] fetching with currentPath:', currentPath.value, '| key:', ['files', serverId.value, currentPath.value])
		await modulesLoaded
		return client.kyros.files_v0.listDirectory(currentPath.value, pageParam, 100)
	},
	getNextPageParam: (lastPage, allPages) => {
		const totalFetched = allPages.reduce((sum, page) => sum + page.items.length, 0)
		return totalFetched < lastPage.total ? allPages.length + 1 : undefined
	},
	staleTime: 30_000,
	initialPageParam: 1,
})

const items = computed(() => directoryData.value?.pages.flatMap((page) => page.items) ?? [])

// prefetch directory contents on hover (150ms debounce)
function prefetchDirectory(path: string) {
	queryClient.prefetchInfiniteQuery({
		queryKey: ['files', serverId.value, path],
		queryFn: async () => {
			await modulesLoaded
			try {
				return await client.kyros.files_v0.listDirectory(path, 1, 100)
			} catch {
				// silently fail - folder may not exist yet (an optimistic update)
				return { items: [], total: 0, current: 1 }
			}
		},
		initialPageParam: 1,
		staleTime: 30_000,
	})
}

let prefetchTimeout: ReturnType<typeof setTimeout> | null = null

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
			console.log('[prefetch] navPath:', navPath)
			prefetchDirectory(navPath)
		}, 150)
	}
}

function getQueryKey() {
	return ['files', serverId.value, currentPath.value]
}

const deleteMutation = useMutation({
	mutationFn: ({ path, recursive }: { path: string; recursive: boolean }) =>
		client.kyros.files_v0.deleteFileOrFolder(path, recursive),

	onMutate: async ({ path }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)

		// optimistically remove the item
		queryClient.setQueryData(queryKey, (old: any) => ({
			...old,
			pages: old?.pages?.map((page: any) => ({
				...page,
				items: page.items.filter((item: any) => item.path !== path),
				total: Math.max(0, page.total - 1),
			})),
		}))
		return { previous }
	},

	onError: (err: any, _vars, context) => {
		queryClient.setQueryData(getQueryKey(), context?.previous)
		addNotification({ title: 'Delete failed', text: err.message, type: 'error' })
	},

	onSuccess: () => {
		addNotification({ title: 'File deleted', text: 'Your file has been deleted.', type: 'success' })
	},

	onSettled: () => {
		queryClient.invalidateQueries({ queryKey: ['files', serverId.value] })
	},
})

const renameMutation = useMutation({
	mutationFn: ({ path, newName }: { path: string; newName: string }) =>
		client.kyros.files_v0.renameFileOrFolder(path, newName),

	onMutate: async ({ path, newName }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)

		// optimistically rename the item
		queryClient.setQueryData(queryKey, (old: any) => ({
			...old,
			pages: old?.pages?.map((page: any) => ({
				...page,
				items: page.items.map((item: any) =>
					item.path === path
						? { ...item, name: newName, path: item.path.replace(/[^/]+$/, newName) }
						: item,
				),
			})),
		}))
		return { previous }
	},

	onError: (err: any, _vars, context) => {
		queryClient.setQueryData(getQueryKey(), context?.previous)
		addNotification({ title: 'Rename failed', text: err.message, type: 'error' })
	},

	onSuccess: (_, { newName }) => {
		addNotification({ title: 'Renamed', text: `Renamed to ${newName}`, type: 'success' })
	},

	onSettled: () => {
		queryClient.invalidateQueries({ queryKey: ['files', serverId.value] })
	},
})

const moveMutation = useMutation({
	mutationFn: ({ source, destination }: { source: string; destination: string }) =>
		client.kyros.files_v0.moveFileOrFolder(source, destination),

	onMutate: async ({ source }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)

		// optimistically remove from current directory
		queryClient.setQueryData(queryKey, (old: any) => ({
			...old,
			pages: old?.pages?.map((page: any) => ({
				...page,
				items: page.items.filter((item: any) => item.path !== source),
				total: Math.max(0, page.total - 1),
			})),
		}))
		return { previous }
	},

	onError: (err: any, _vars, context) => {
		queryClient.setQueryData(getQueryKey(), context?.previous)
		addNotification({ title: 'Move failed', text: err.message, type: 'error' })
	},

	onSuccess: (_, { destination }) => {
		addNotification({ title: 'Moved', text: `Moved to ${destination}`, type: 'success' })
	},

	onSettled: () => {
		queryClient.invalidateQueries({ queryKey: ['files', serverId.value] })
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

		// optimistically add new item
		const newItem = {
			name,
			path,
			type,
			modified: now,
			created: now,
			...(type === 'directory' ? { count: 0 } : { size: 0 }),
		}
		queryClient.setQueryData(queryKey, (old: any) => ({
			...old,
			pages: old?.pages?.map((page: any, i: number) =>
				i === 0
					? {
							...page,
							items: [newItem, ...page.items],
							total: page.total + 1,
						}
					: page,
			),
		}))
		return { previous }
	},

	onError: (err: any, _vars, context) => {
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
		queryClient.invalidateQueries({ queryKey: ['files', serverId.value] })
	},
})

// (no optimistic update - uses ws for progress)
const extractMutation = useMutation({
	mutationFn: ({ path, override }: { path: string; override: boolean }) =>
		client.kyros.files_v0.extractFile(path, override, false),

	onSuccess: () => {
		addNotification({ title: 'Extraction started', type: 'success' })
	},

	onError: (err: any) => {
		addNotification({ title: 'Extract failed', text: err.message, type: 'error' })
	},

	onSettled: () => {
		queryClient.invalidateQueries({ queryKey: ['files', serverId.value] })
	},
})

function refreshList() {
	queryClient.invalidateQueries({ queryKey: ['files', serverId.value] })
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
	const path = `${currentPath.value}/${selectedItem.value.name}`.replace('//', '/')
	const item = selectedItem.value

	renameMutation.mutate(
		{ path, newName },
		{
			onSuccess: async () => {
				// track for undo
				redoStack.value = []
				operationHistory.value.push({
					type: 'rename',
					itemType: item.type,
					fileName: item.name,
					path: currentPath.value,
					oldName: item.name,
					newName,
				})

				if (closeEditor.value) {
					await props.server.refresh()
					isEditing.value = false
					editingFile.value = null
					closeEditor.value = false
					router.push({ query: { ...route.query, path: currentPath.value } })
				}
			},
		},
	)
}

function extractItem(path: string) {
	// add to queued ops for UI feedback
	fsQueuedOps.value.push({ op: 'unarchive', src: path })
	setTimeout(() => {
		fsQueuedOps.value = fsQueuedOps.value.filter((x) => x.op !== 'unarchive' || x.src !== path)
	}, 4000)

	extractMutation.mutate(
		{ path, override: true },
		{
			onError: () => {
				fsQueuedOps.value = fsQueuedOps.value.filter((x) => x.op !== 'unarchive' || x.src !== path)
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
				uploadConflictModal.value.show(item.path, dry.conflicting_files)
			}
		} else {
			handleServersError(new Error('Error running dry run'), notifications)
		}
	} catch (error) {
		console.error('Error extracting item:', error)
		handleServersError(error, notifications)
	}
}

function handleMoveItem(destination: string) {
	const item = selectedItem.value
	const sourcePath = currentPath.value
	const source = `${sourcePath}/${item.name}`.replace('//', '/')
	const dest = `${destination}/${item.name}`.replace('//', '/')

	moveMutation.mutate(
		{ source, destination: dest },
		{
			onSuccess: () => {
				// track for undo
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
				// track for undo
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
	const path = `${currentPath.value}/${selectedItem.value.name}`.replace('//', '/')
	deleteMutation.mutate({ path, recursive: selectedItem.value.type === 'directory' })
}

function showCreateModal(type: 'file' | 'directory') {
	newItemType.value = type
	createItemModal.value?.show()
}

function showUnzipFromUrlModal(cf: boolean) {
	uploadZipModal.value?.show(cf)
}

function showRenameModal(item: any) {
	selectedItem.value = item
	renameItemModal.value?.show(item)
	contextMenuInfo.value.item = null
}

function showMoveModal(item: any) {
	selectedItem.value = item
	moveItemModal.value?.show()
	contextMenuInfo.value.item = null
}

function showDeleteModal(item: any) {
	selectedItem.value = item
	deleteItemModal.value?.show()
	contextMenuInfo.value.item = null
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
				// modified/created are Unix timestamps (seconds), compare directly
				return sortDesc.value ? a.modified - b.modified : b.modified - a.modified
			case 'created':
				return sortDesc.value ? a.created - b.created : b.created - a.created
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

function onInit(editor: any) {
	editor.commands.addCommand({
		name: 'saveFile',
		bindKey: { win: 'Ctrl-S', mac: 'Command-S' },
		exec: () => saveFileContent(false),
	})
}

async function showContextMenu(item: any, x: number, y: number) {
	contextMenuInfo.value = { item, x, y }
	selectedItem.value = item
	await nextTick()
	if (!contextMenu.value?.ctxRef) return false
	const screenHeight = window.innerHeight
	const ctxRect = contextMenu.value.ctxRef.getBoundingClientRect()
	isAtBottom.value = ctxRect.bottom > screenHeight
}

function onAnywhereClicked(e: MouseEvent) {
	if (!(e.target as HTMLElement).closest('#item-context-menu')) {
		contextMenuInfo.value.item = null
	}
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

const imageExtensions = ['png', 'jpg', 'jpeg', 'gif', 'webp']

async function editFile(item: { name: string; type: string; path: string }) {
	try {
		const path = `${currentPath.value}/${item.name}`.replace('//', '/')
		const content = await client.kyros.files_v0.downloadFile(path)
		window.scrollTo(0, 0)

		const extension = item.name.split('.').pop()?.toLowerCase()
		editingFile.value = item
		isEditing.value = true

		// check if image before consuming the blob
		if (item.type === 'file' && extension && imageExtensions.includes(extension)) {
			isEditingImage.value = true
			imagePreview.value = content
		} else {
			isEditingImage.value = false
			fileContent.value = await content.text()
		}
		router.push({ query: { ...route.query, path: currentPath.value, editing: item.path } })
	} catch (error) {
		console.error('Error fetching file content:', error)
		addNotification({ title: 'Failed to open file', text: 'Could not load file contents.', type: 'error' })
	}
}

async function initializeFileEdit() {
	if (!route.query.editing) return

	const filePath = route.query.editing as string
	await editFile({
		name: filePath.split('/').pop() || '',
		type: 'file',
		path: filePath,
	})
}

onMounted(async () => {
	await modulesLoaded

	await initializeFileEdit()

	await import('ace-builds')
	await import('ace-builds/src-noconflict/mode-json')
	await import('ace-builds/src-noconflict/mode-yaml')
	await import('ace-builds/src-noconflict/mode-toml')
	await import('ace-builds/src-noconflict/mode-sh')
	await import('ace-builds/src-noconflict/theme-one_dark')
	await import('ace-builds/src-noconflict/ext-searchbox')
	VAceEditor.value = markRaw((await import('vue3-ace-editor')).VAceEditor)
	document.addEventListener('click', onAnywhereClicked)
	window.addEventListener('scroll', onScroll)

	document.addEventListener('keydown', onKeydown)

	fsQueuedOps.value = []
})

onUnmounted(() => {
	document.removeEventListener('click', onAnywhereClicked)
	window.removeEventListener('scroll', onScroll)
	document.removeEventListener('keydown', onKeydown)
})

type QueuedOpWithState = FSQueuedOp & { state: 'queued' }

const ops = computed<(QueuedOpWithState | FilesystemOp)[]>(() => [
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
	() => route.query,
	async (newQuery, oldQuery) => {
		if (newQuery.path !== oldQuery?.path) {
			searchQuery.value = ''
			viewFilter.value = 'all'
			sortMethod.value = 'name'
			sortDesc.value = false
		}

		if (newQuery.editing && editingFile.value?.path !== newQuery.editing) {
			await editFile({
				name: (newQuery.editing as string).split('/').pop() || '',
				type: 'file',
				path: newQuery.editing as string,
			})
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
	const newPath = breadcrumbSegments.value.slice(0, index + 1).join('/')
	router.push({ query: { ...route.query, path: newPath } })
	if (isEditing.value) {
		isEditing.value = false
		editingFile.value = null
		closeEditor.value = false

		const newQuery = { ...route.query }
		delete newQuery.editing
		router.replace({ query: newQuery })
	}
}

// const navigateToPage = () => {
//   router.push({ query: { path: currentPath.value } });
// };

async function requestShareLink() {
	try {
		const response = (await $fetch('https://api.mclo.gs/1/log', {
			method: 'POST',
			headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
			body: new URLSearchParams({ content: fileContent.value }),
		})) as any

		if (response.success) {
			await navigator.clipboard.writeText(response.url)
			addNotification({
				title: 'Log URL copied',
				text: 'Your log file URL has been copied to your clipboard.',
				type: 'success',
			})
		} else {
			throw new Error(response.error)
		}
	} catch (error) {
		console.error('Error sharing file:', error)
		addNotification({ title: 'Failed to share file', text: 'Could not upload to mclo.gs.', type: 'error' })
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

async function downloadFile(item: any) {
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
			addNotification({ title: 'Download failed', text: 'Could not download the file.', type: 'error' })
		}
		contextMenuInfo.value.item = null
	}
}

async function saveFileContent(exit: boolean = true) {
	if (!editingFile.value) return

	try {
		await client.kyros.files_v0.updateFile(editingFile.value.path, fileContent.value)
		if (exit) {
			await props.server.refresh()
			isEditing.value = false
			editingFile.value = null
			router.push({ query: { ...route.query, path: currentPath.value } })
		}

		addNotification({
			title: 'File saved',
			text: 'Your file has been saved.',
			type: 'success',
		})
	} catch (error) {
		console.error('Error saving file content:', error)
		addNotification({ title: 'Save failed', text: 'Could not save the file.', type: 'error' })
	}
}

async function saveFileContentRestart() {
	await saveFileContent()
	await props.server.general?.power('Restart')

	addNotification({
		title: 'Server restarted',
		text: 'Your server has been restarted.',
		type: 'success',
	})
}

async function saveFileContentAs() {
	await saveFileContent(false)
	closeEditor.value = true
	showRenameModal(editingFile.value)
}

function cancelEditing() {
	isEditing.value = false
	editingFile.value = null
	fileContent.value = ''
	isEditingImage.value = false
	imagePreview.value = null
	router.push({ query: { ...route.query, path: currentPath.value } })
	const newQuery = { ...route.query }
	delete newQuery.editing
	router.replace({ query: newQuery })
}

function onScroll() {
	if (contextMenuInfo.value.item) {
		contextMenuInfo.value.y = Math.max(0, contextMenuInfo.value.y - window.scrollY)
	}
}
</script>

<style scoped>
.upload-status {
	overflow: hidden;
	transition: height 0.2s ease;
}

.upload-status-enter-active,
.upload-status-leave-active {
	transition: height 0.2s ease;
	overflow: hidden;
}

.upload-status-enter-from,
.upload-status-leave-to {
	height: 0 !important;
}

.status-icon-enter-active,
.status-icon-leave-active {
	transition: all 0.25s ease;
}

.status-icon-enter-from,
.status-icon-leave-to {
	transform: scale(0);
	opacity: 0;
}

.status-icon-enter-to,
.status-icon-leave-from {
	transform: scale(1);
	opacity: 1;
}

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
</style>
