<script setup lang="ts">
import type { Kyros } from '@modrinth/api-client'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ReadyTransition from '#ui/components/base/ReadyTransition.vue'
import { useReadyState } from '#ui/composables'
import { useUploadSessionUpload } from '#ui/composables/hosting/kyros-session-upload'
import { useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import type { EditingFile, FileItem } from '#ui/layouts/shared/files-tab/index.ts'
import { FilePageLayout, provideFileManager } from '#ui/layouts/shared/files-tab/index.ts'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'
import { canOpenInFileEditor } from '#ui/utils/file-extensions'

const props = defineProps<{
	showDebugInfo?: boolean
	showRefreshButton?: boolean
}>()

const client = injectModrinthClient()
const serverContext = injectModrinthServerContext()
const {
	serverId,
	worldId,
	fsOps,
	busyReasons,
	uploadState,
	cancelUpload: cancelUploadRef,
} = serverContext
const fileUploadSession = useUploadSessionUpload({
	client,
	scope: 'files',
	worldId,
	uploadState,
	cancelUpload: cancelUploadRef,
})
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const { canWriteFiles, canUsePowerActions, permissionDeniedMessage } = useServerPermissions()

const route = useRoute()
const router = useRouter()
const queryClient = useQueryClient()

const serverBusy = computed(() => busyReasons.value.length > 0)
const busyTooltip = computed(() =>
	busyReasons.value.length > 0 ? formatMessage(busyReasons.value[0].reason) : undefined,
)
const fileWriteDisabled = computed(() => !canWriteFiles.value || serverBusy.value)
const fileWriteDisabledTooltip = computed(() =>
	canWriteFiles.value ? busyTooltip.value : permissionDeniedMessage.value,
)
const nonBackupBusyReasons = computed(() =>
	busyReasons.value.filter(
		(r) =>
			r.reason.id !== 'servers.busy.backup-creating' &&
			r.reason.id !== 'servers.busy.backup-restoring',
	),
)

const busyWarning = computed(() =>
	nonBackupBusyReasons.value.length > 0
		? formatMessage(nonBackupBusyReasons.value[0].reason)
		: null,
)

// Path & navigation
const currentPath = computed(() => (typeof route.query.path === 'string' ? route.query.path : '/'))

function navigateTo(path: string) {
	const { editing: _, ...query } = route.query
	router.push({ query: { ...query, path } })
}

// Editing state (synced with URL)
const editingFile = ref<EditingFile | null>(null)

function startEditing(file: EditingFile) {
	editingFile.value = file
	router.push({ query: { ...route.query, path: currentPath.value, editing: file.path } })
}

function stopEditing() {
	editingFile.value = null
	const newQuery = { ...route.query }
	delete newQuery.editing
	router.replace({ query: newQuery })
}

// Sync editing state from URL
watch(
	() => route.query,
	(newQuery, oldQuery) => {
		if (newQuery.editing && editingFile.value?.path !== newQuery.editing) {
			editingFile.value = {
				name: (newQuery.editing as string).split('/').pop() || '',
				path: newQuery.editing as string,
			}
		} else if (oldQuery?.editing && !newQuery.editing) {
			editingFile.value = null
		}
	},
	{ deep: true },
)

// Initialize editing from URL on mount
function initializeFileEdit() {
	if (!route.query.editing) return
	const filePath = route.query.editing as string
	editingFile.value = {
		name: filePath.split('/').pop() || '',
		path: filePath,
	}
}

function getWorldId() {
	if (!worldId.value) {
		throw new Error('World ID is not available.')
	}
	return worldId.value
}

function timestampSeconds(value: string) {
	return Math.floor(new Date(value).getTime() / 1000)
}

function toFileItem(item: Kyros.Files.v1.FileListingItem): FileItem | null {
	if (item.type === 'other') return null

	return {
		name: item.name,
		type: item.type === 'regular' ? 'file' : item.type,
		path: item.full_path,
		modified: timestampSeconds(item.mtime),
		created: timestampSeconds(item.ctime),
		...(item.type === 'directory' ? { count: item.descendants } : { size: item.size_bytes }),
	}
}

// Directory listing query
const {
	data: directoryData,
	isLoading,
	error: loadError,
} = useQuery({
	queryKey: computed(() => ['files', 'v1', worldId.value, currentPath.value]),
	queryFn: async () => {
		return client.kyros.files_v1.listDescendants(getWorldId(), currentPath.value, 1, 200)
	},
	enabled: computed(() => !!worldId.value),
	staleTime: 30_000,
})

function isVisibleFileItem(item: FileItem) {
	return !item.path.split('/').includes('.modrinth-staged')
}

const items = computed<FileItem[]>(() =>
	(directoryData.value?.items ?? [])
		.map(toFileItem)
		.filter((item): item is FileItem => !!item)
		.filter(isVisibleFileItem),
)

const filesReadyPending = useReadyState({ isLoading, data: directoryData })

// Prefetching
function prefetchDirectory(path: string) {
	const id = worldId.value
	if (!id) return

	queryClient.prefetchQuery({
		queryKey: ['files', 'v1', id, path],
		queryFn: async () => {
			try {
				return await client.kyros.files_v1.listDescendants(id, path, 1, 200)
			} catch {
				return {
					items: [],
					page: 1,
					items_per_page: 200,
					page_total: 0,
					items_total: 0,
					too_many_descendants: false,
					descendants_limit: 0,
				}
			}
		},
		staleTime: 30_000,
	})
}

function prefetchFile(path: string) {
	const id = worldId.value
	if (!id) return
	if (!canOpenInFileEditor(path.split('/').pop() ?? path)) return

	queryClient.prefetchQuery({
		queryKey: ['file-content', id, path],
		queryFn: async () => {
			try {
				const blob = await client.kyros.files_v1.downloadRawFileContents(id, path)
				return await blob.text()
			} catch {
				return null
			}
		},
		staleTime: 30_000,
	})
}

function getQueryKey() {
	return ['files', 'v1', worldId.value, currentPath.value]
}

function refreshList() {
	queryClient.invalidateQueries({ queryKey: ['files', 'v1', worldId.value] })
}

// Mutations
const deleteMutation = useMutation({
	mutationFn: ({ path }: { path: string; recursive: boolean }) =>
		client.kyros.files_v1.deleteFile(getWorldId(), path),
	onMutate: async ({ path }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)
		queryClient.setQueryData(queryKey, (old: Kyros.Files.v1.FileListingResponse | undefined) => {
			if (!old) return old
			return { ...old, items: old.items.filter((item) => item.full_path !== path) }
		})
		return { previous }
	},
	onError: (err: Error, _vars, context) => {
		queryClient.setQueryData(getQueryKey(), context?.previous)
		addNotification({
			title: formatMessage(commonMessages.deleteFailedLabel),
			text: err.message,
			type: 'error',
		})
	},
	onSuccess: () => {
		addNotification({
			title: 'File deleted',
			text: 'Your file has been deleted.',
			type: 'success',
		})
	},
	onSettled: () => {
		refreshList()
	},
})

const renameMutation = useMutation({
	mutationFn: ({ path, newName }: { path: string; newName: string }) =>
		client.kyros.files_v1.renameFile(getWorldId(), path, newName),
	onMutate: async ({ path, newName }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)
		queryClient.setQueryData(queryKey, (old: Kyros.Files.v1.FileListingResponse | undefined) => {
			if (!old) return old
			return {
				...old,
				items: old.items.map((item) =>
					item.full_path === path
						? {
								...item,
								name: newName,
								full_path: item.full_path.replace(/[^/]+$/, newName),
							}
						: item,
				),
			}
		})
		return { previous }
	},
	onError: (err: Error, _vars, context) => {
		queryClient.setQueryData(getQueryKey(), context?.previous)
		addNotification({
			title: formatMessage(commonMessages.renameFailedLabel),
			text: err.message,
			type: 'error',
		})
	},
	onSuccess: (_, { newName }) => {
		addNotification({ title: 'Renamed', text: `Renamed to ${newName}`, type: 'success' })
	},
	onSettled: () => {
		refreshList()
	},
})

const moveMutation = useMutation({
	mutationFn: ({ source, destination }: { source: string; destination: string }) =>
		client.kyros.files_v1.moveFile(getWorldId(), source, destination),
	onMutate: async ({ source }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)
		queryClient.setQueryData(queryKey, (old: Kyros.Files.v1.FileListingResponse | undefined) => {
			if (!old) return old
			return { ...old, items: old.items.filter((item) => item.full_path !== source) }
		})
		return { previous }
	},
	onError: (err: Error, _vars, context) => {
		queryClient.setQueryData(getQueryKey(), context?.previous)
		addNotification({
			title: formatMessage(commonMessages.moveFailedLabel),
			text: err.message,
			type: 'error',
		})
	},
	onSuccess: (_, { destination }) => {
		addNotification({ title: 'Moved', text: `Moved to ${destination}`, type: 'success' })
	},
	onSettled: () => {
		refreshList()
	},
})

const createMutation = useMutation({
	mutationFn: ({ path, type }: { path: string; type: 'file' | 'directory' }) => {
		const id = getWorldId()
		return type === 'directory'
			? client.kyros.files_v1.mkdirFile(id, path)
			: client.kyros.files_v1.touchFile(id, path)
	},
	onError: (err: Error) => {
		addNotification({
			title: formatMessage(commonMessages.createFailedLabel),
			text: err.message,
			type: 'error',
		})
	},
	onSettled: () => {
		refreshList()
	},
})

// File I/O
async function readFile(path: string): Promise<string> {
	const normalizedPath = path.startsWith('/') ? path : `/${path}`
	const id = getWorldId()
	const cachedContent = queryClient.getQueryData<string>(['file-content', id, normalizedPath])
	if (cachedContent != null) return cachedContent
	const blob = await client.kyros.files_v1.downloadRawFileContents(id, normalizedPath)
	return await blob.text()
}

async function readFileAsBlob(path: string): Promise<Blob> {
	const normalizedPath = path.startsWith('/') ? path : `/${path}`
	return await client.kyros.files_v1.downloadRawFileContents(getWorldId(), normalizedPath)
}

async function writeFile(path: string, content: string): Promise<void> {
	if (fileWriteDisabled.value) return
	const normalizedPath = path.startsWith('/') ? path : `/${path}`
	const id = getWorldId()
	await client.kyros.files_v1.editFile(id, normalizedPath, content)
	queryClient.setQueryData(['file-content', id, normalizedPath], content)
	refreshList()
}

async function downloadFile(path: string, fileName: string): Promise<void> {
	try {
		const fileData = await client.kyros.files_v1.downloadRawFileContents(getWorldId(), path)
		if (fileData) {
			const blob = new Blob([fileData], { type: 'application/octet-stream' })
			const link = document.createElement('a')
			link.href = window.URL.createObjectURL(blob)
			link.download = fileName
			link.click()
			window.URL.revokeObjectURL(link.href)
		}
	} catch {
		addNotification({
			title: formatMessage(commonMessages.downloadFailedLabel),
			text: 'Could not download the file.',
			type: 'error',
		})
	}
}

watch(
	() => fsOps.value,
	() => {
		refreshList()
	},
)

onMounted(async () => {
	initializeFileEdit()
})

// Restart
async function restartServer() {
	if (!canUsePowerActions.value || !worldId.value) return
	await client.archon.servers_v1.powerWorld(serverId, worldId.value, { action: 'restart' })
}

function getSessionUploadFilename(fileName: string) {
	const basePath = currentPath.value.split('/').filter(Boolean).join('/')
	return basePath ? `${basePath}/${fileName}` : fileName
}

async function uploadFiles(files: File[]) {
	if (fileWriteDisabled.value || files.length === 0) return

	try {
		const result = await fileUploadSession.uploadFiles(
			files.map((file) => ({
				file,
				filename: getSessionUploadFilename(file.name),
			})),
		)
		if (result === 'completed') refreshList()
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.uploadFailedLabel),
			text: err instanceof Error ? err.message : undefined,
			type: 'error',
		})
	}
}

function cancelUpload() {
	fileUploadSession.cancelUpload()
}

// Provide the file manager context
provideFileManager({
	items,
	loading: computed(() => isLoading.value),
	error: computed(() => loadError.value ?? null),
	currentPath,
	navigateTo,
	editingFile,
	startEditing,
	stopEditing,
	createItem: async (name, type) => {
		if (fileWriteDisabled.value) return
		const path = `${currentPath.value}/${name}`.replace('//', '/')
		await createMutation.mutateAsync({ path, type })
	},
	renameItem: async (path, newName) => {
		if (fileWriteDisabled.value) return
		await renameMutation.mutateAsync({ path, newName })
	},
	moveItem: async (source, destination) => {
		if (fileWriteDisabled.value) return
		await moveMutation.mutateAsync({ source, destination })
	},
	deleteItem: async (path, recursive) => {
		if (fileWriteDisabled.value) return
		await deleteMutation.mutateAsync({ path, recursive })
	},
	readFile,
	readFileAsBlob,
	writeFile,
	downloadFile,
	uploadFiles,
	cancelUpload,
	uploadState,
	worldId,
	refresh: refreshList,
	isBusy: fileWriteDisabled,
	busyTooltip: fileWriteDisabledTooltip,
	busyWarning,
	prefetchDirectory,
	prefetchFile,
	showInstallFromUrl: true,
	canRestart: canUsePowerActions.value,
	restartServer,
	canShareToMclogs: true,
})
</script>

<template>
	<ReadyTransition :pending="filesReadyPending">
		<FilePageLayout
			:show-debug-info="props.showDebugInfo"
			:show-refresh-button="props.showRefreshButton"
		/>
	</ReadyTransition>
</template>
