<script setup lang="ts">
import type { Kyros } from '@modrinth/api-client'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ReadyTransition from '#ui/components/base/ReadyTransition.vue'
import { useReadyState } from '#ui/composables'
import { useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

import FilePageLayout from '../../../shared/files-tab/layout.vue'
import { provideFileManager } from '../../../shared/files-tab/providers/file-manager'
import type { EditingFile, FileItem } from '../../../shared/files-tab/types'

const props = defineProps<{
	showDebugInfo?: boolean
	showRefreshButton?: boolean
}>()

const client = injectModrinthClient()
const serverContext = injectModrinthServerContext()
const { serverId, fsOps, busyReasons, uploadState, cancelUpload: cancelUploadRef } = serverContext
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const route = useRoute()
const router = useRouter()
const queryClient = useQueryClient()

const serverBusy = computed(() => busyReasons.value.length > 0)
const busyTooltip = computed(() =>
	busyReasons.value.length > 0 ? formatMessage(busyReasons.value[0].reason) : undefined,
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

// Directory listing query
const {
	data: directoryData,
	isLoading,
	error: loadError,
} = useQuery({
	queryKey: computed(() => ['files', serverId, currentPath.value]),
	queryFn: async () => {
		return client.kyros.files_v0.listDirectory(currentPath.value, 1, 2000)
	},
	staleTime: 30_000,
})

const items = computed<FileItem[]>(() => directoryData.value?.items ?? [])

const filesReadyPending = useReadyState({ isLoading, data: directoryData })

// Prefetching
function prefetchDirectory(path: string) {
	queryClient.prefetchQuery({
		queryKey: ['files', serverId, path],
		queryFn: async () => {
			try {
				return await client.kyros.files_v0.listDirectory(path, 1, 2000)
			} catch {
				return { items: [], total: 0, current: 1 }
			}
		},
		staleTime: 30_000,
	})
}

function prefetchFile(path: string) {
	queryClient.prefetchQuery({
		queryKey: ['file-content', serverId, path],
		queryFn: async () => {
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

function getQueryKey() {
	return ['files', serverId, currentPath.value]
}

function refreshList() {
	queryClient.invalidateQueries({ queryKey: ['files', serverId] })
}

// Mutations
const deleteMutation = useMutation({
	mutationFn: ({ path, recursive }: { path: string; recursive: boolean }) =>
		client.kyros.files_v0.deleteFileOrFolder(path, recursive),
	onMutate: async ({ path }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)
		queryClient.setQueryData(queryKey, (old: Kyros.Files.v0.DirectoryResponse | undefined) => {
			if (!old) return old
			return { ...old, items: old.items.filter((item) => item.path !== path) }
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
		queryClient.setQueryData(queryKey, (old: Kyros.Files.v0.DirectoryResponse | undefined) => {
			if (!old) return old
			return {
				...old,
				items: old.items.map((item) =>
					item.path === path
						? {
								...item,
								name: newName,
								path: item.path.replace(/[^/]+$/, newName),
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
		queryClient.setQueryData(queryKey, (old: Kyros.Files.v0.DirectoryResponse | undefined) => {
			if (!old) return old
			return { ...old, items: old.items.filter((item) => item.path !== source) }
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
		queryClient.setQueryData(queryKey, (old: Kyros.Files.v0.DirectoryResponse | undefined) => {
			if (!old) return old
			return { ...old, items: [newItem, ...old.items] }
		})
		return { previous }
	},
	onError: (err: Error, _vars, context) => {
		queryClient.setQueryData(getQueryKey(), context?.previous)
		addNotification({
			title: formatMessage(commonMessages.createFailedLabel),
			text: err.message,
			type: 'error',
		})
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

// Extraction
async function extractFile(path: string, override: boolean, dry: boolean) {
	if (dry) {
		return await client.kyros.files_v0.extractFile(path, override, true)
	}
	await client.kyros.files_v0.extractFile(path, override, false)
}

// File I/O
async function readFile(path: string): Promise<string> {
	const normalizedPath = path.startsWith('/') ? path : `/${path}`
	const cachedContent = queryClient.getQueryData<string>(['file-content', serverId, normalizedPath])
	if (cachedContent) return cachedContent
	const blob = await client.kyros.files_v0.downloadFile(normalizedPath)
	return await blob.text()
}

async function readFileAsBlob(path: string): Promise<Blob> {
	const normalizedPath = path.startsWith('/') ? path : `/${path}`
	return await client.kyros.files_v0.downloadFile(normalizedPath)
}

async function writeFile(path: string, content: string): Promise<void> {
	await client.kyros.files_v0.updateFile(path, content)
	queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
}

async function downloadFile(path: string, fileName: string): Promise<void> {
	try {
		const fileData = await client.kyros.files_v0.downloadFile(path)
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
	await client.archon.servers_v0.power(serverId, 'Restart')
}

let activeUploadCancel: (() => void) | null = null

async function uploadFiles(files: File[]) {
	if (files.length === 0) return

	const totalBytes = files.reduce((sum, f) => sum + f.size, 0)
	uploadState.value = {
		isUploading: true,
		currentFileName: files[0].name,
		currentFileProgress: 0,
		uploadedBytes: 0,
		totalBytes,
		completedFiles: 0,
		totalFiles: files.length,
	}
	cancelUploadRef.value = () => activeUploadCancel?.()

	let completedBytes = 0

	for (let i = 0; i < files.length; i++) {
		const file = files[i]
		const filePath = `${currentPath.value}/${file.name}`.replace('//', '/')

		uploadState.value.currentFileName = file.name
		uploadState.value.currentFileProgress = 0

		try {
			const uploader = client.kyros.files_v0.uploadFile(filePath, file, {
				onProgress: ({ progress }) => {
					uploadState.value.currentFileProgress = progress
					uploadState.value.uploadedBytes = completedBytes + Math.round(file.size * progress)
				},
			})
			activeUploadCancel = () => uploader.cancel()

			await uploader.promise
			completedBytes += file.size
			uploadState.value.completedFiles = i + 1
			uploadState.value.uploadedBytes = completedBytes
		} catch (err) {
			if (err instanceof Error && err.message === 'Upload cancelled') break
			addNotification({
				title: formatMessage(commonMessages.uploadFailedLabel),
				text: `Failed to upload ${file.name}`,
				type: 'error',
			})
		}
	}

	activeUploadCancel = null
	cancelUploadRef.value = null
	refreshList()
	uploadState.value = {
		isUploading: false,
		currentFileName: null,
		currentFileProgress: 0,
		uploadedBytes: 0,
		totalBytes: 0,
		completedFiles: 0,
		totalFiles: 0,
	}
}

function cancelUpload() {
	activeUploadCancel?.()
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
		const path = `${currentPath.value}/${name}`.replace('//', '/')
		await createMutation.mutateAsync({ path, type })
	},
	renameItem: async (path, newName) => {
		await renameMutation.mutateAsync({ path, newName })
	},
	moveItem: async (source, destination) => {
		await moveMutation.mutateAsync({ source, destination })
	},
	deleteItem: async (path, recursive) => {
		await deleteMutation.mutateAsync({ path, recursive })
	},
	readFile,
	readFileAsBlob,
	writeFile,
	downloadFile,
	uploadFiles,
	cancelUpload,
	uploadState,
	refresh: refreshList,
	isBusy: serverBusy,
	busyTooltip,
	busyWarning,
	extractFile,
	prefetchDirectory,
	prefetchFile,
	showInstallFromUrl: true,
	canRestart: true,
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
