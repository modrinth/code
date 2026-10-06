<script setup lang="ts">
import type {
	DirectoryResult,
	DirectoryTree,
	ExtractDryRunResult,
	FileInfo,
	FileItem,
	FileItemResultFrom,
	FileResult,
	FileTypes,
	UploadState,
} from '@modrinth/ui'
import {
	childPath,
	commonMessages,
	defineMessages,
	FilePageLayout,
	infoFrom,
	injectNotificationManager,
	normalizeDirectoryPath,
	parentInfoFrom,
	provideFileManager,
	ReadyTransition,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { invoke } from '@tauri-apps/api/core'
import { computed, effectScope, markRaw, onScopeDispose, ref, shallowReactive, watch } from 'vue'

import { useAppEvent } from '@/composables/use-app-event'
import { get_full_path } from '@/helpers/instance'
import { highlightInFolder } from '@/helpers/utils'

import { injectInstancePage } from '../instance-context'
import { instanceKeys } from '../query-options'

const instancePage = injectInstancePage()
const instanceId = instancePage.instanceId
const queryClient = useQueryClient()

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const debug = useDebugLogger('Files')

const messages = defineMessages({
	readOnly: {
		id: 'instance.files.managed-content-read-only',
		defaultMessage: 'Manage your installed content via the Content tab',
	},
	saveAs: {
		id: 'instance.files.save-as',
		defaultMessage: 'Save as...',
	},
	addingFiles: {
		id: 'instance.files.adding-files',
		defaultMessage: 'Adding files ({completed}/{total})',
	},
})

const instanceRootQuery = useQuery(
	computed(() => ({
		queryKey: instanceKeys.rootPath(instancePage.instanceId.value),
		queryFn: () => get_full_path(instancePage.instanceId.value),
		enabled: !!instancePage.instanceId.value,
		staleTime: Infinity,
	})),
)
const instanceRoot = computed(() => instanceRootQuery.data.value ?? '')

const currentLocation = ref<FileInfo>(infoFrom('/'))
const currentDirectory = computed<FileInfo<'directory'>>(() =>
	currentLocation.value.type === 'file'
		? parentInfoFrom(currentLocation.value)
		: (currentLocation.value as FileInfo<'directory'>),
)

debug('setup: start, instance.id =', instanceId.value)

/** The instance's file commands take paths relative to the instance root, without a leading slash. */
function toRelativePath(path: string) {
	return path.split('/').filter(Boolean).join('/')
}

async function listDirectory(path: string): Promise<FileItem[]> {
	const items = await invoke<FileItem[]>('plugin:files|file_list', {
		instanceId: instanceId.value,
		path: toRelativePath(path),
	})
	return items.map((item) => ({ ...item, path: normalizeDirectoryPath(item.path) }))
}

async function readFile(path: string): Promise<ArrayBuffer> {
	const bytes = await invoke<number[]>('plugin:files|file_read', {
		instanceId: instanceId.value,
		path: toRelativePath(path),
	})
	return markRaw(new Uint8Array(bytes).buffer)
}

async function writeBytes(path: string, bytes: Uint8Array, createOnly = false) {
	await invoke('plugin:files|file_write', {
		instanceId: instanceId.value,
		path: toRelativePath(path),
		bytes: Array.from(bytes),
		createOnly,
	})
}

function directoryQueryOptions(path: string) {
	return {
		queryKey: instanceKeys.files(instanceId.value, path),
		queryFn: () => listDirectory(path),
		staleTime: 30_000,
	}
}

function fileQueryOptions(path: string) {
	return {
		queryKey: instanceKeys.fileContent(instanceId.value, path),
		queryFn: () => readFile(path),
		staleTime: 30_000,
	}
}

function queryDirectory(info: FileInfo<'directory'>): DirectoryResult {
	const query = useQuery(
		computed(() => ({ ...directoryQueryOptions(info.path), enabled: !!instanceRoot.value })),
		queryClient,
	)
	return {
		...info,
		data: computed(() => query.data.value ?? []),
		isLoading: query.isLoading,
		loadError: query.error,
	}
}

function queryFile(info: FileInfo<'file'>): FileResult {
	const query = useQuery(
		computed(() => ({ ...fileQueryOptions(info.path), enabled: !!instanceRoot.value })),
		queryClient,
	)
	return {
		...info,
		data: computed(() => query.data.value ?? null),
		isLoading: query.isLoading,
		loadError: query.error,
	}
}

const fileQueries = shallowReactive(new Map<string, DirectoryResult | FileResult>())
const expandedDirectories = ref<string[]>([])

/** Owns the lazily created queries so they're disposed with this page, wherever they were first requested from. */
const fileSystemScope = effectScope()
onScopeDispose(() => fileSystemScope.stop())

function queryCacheKey(info: FileInfo) {
	return `${info.type === 'file' ? 'file' : 'directory'}:${normalizeDirectoryPath(info.path)}`
}

const directoryTree = {
	get: <T extends FileTypes>(info: FileInfo<T>): FileItemResultFrom<T> => {
		const key = queryCacheKey(info)
		let entry = fileQueries.get(key)
		if (!entry) {
			entry = fileSystemScope.run(() =>
				info.type === 'file'
					? queryFile(info as FileInfo<'file'>)
					: queryDirectory(info as FileInfo<'directory'>),
			)!
			fileQueries.set(key, entry)
		}
		return entry as FileItemResultFrom<T>
	},
	prefetch: <T extends FileTypes>(info: FileInfo<T>) => {
		if (fileQueries.has(queryCacheKey(info))) return
		queryClient.prefetchQuery(
			info.type === 'file' ? fileQueryOptions(info.path) : directoryQueryOptions(info.path),
		)
	},
	expandedEntries: expandedDirectories,
} satisfies DirectoryTree

/** Paths of every loaded entry marked read-only, so lookups don't rescan all listings per row. */
const readOnlyPaths = computed(() => {
	const paths = new Set<string>()
	for (const entry of fileQueries.values()) {
		if (entry.type !== 'directory') continue
		for (const item of (entry as DirectoryResult).data.value) {
			if (item.readOnly) paths.add(item.path)
		}
	}
	return paths
})

function isReadOnly(file: FileInfo | null): boolean {
	// TODO: HANDLE NULL BY CHECKING IF THE USER CAN CREATE FILES
	if (file == null) return false
	if (toRelativePath(file.path).split('/')[0].toLowerCase() === 'mods') return true
	return readOnlyPaths.value.has(normalizeDirectoryPath(file.path))
}

const isRefreshing = ref<boolean>(false)

async function refresh() {
	debug('refresh: called, currentDirectory =', currentDirectory.value.path)
	isRefreshing.value = true
	try {
		await queryClient.invalidateQueries({
			queryKey: [...instanceKeys.detail(instanceId.value), 'files'],
		})
	} finally {
		isRefreshing.value = false
	}
}

function navigateTo(file: FileInfo) {
	debug('navigateTo:', file.path)
	currentLocation.value = file
}

function notifyError(title: string, error: unknown) {
	addNotification({
		title,
		text: error instanceof Error ? error.message : String(error ?? ''),
		type: 'error',
	})
}

async function createItem(name: string, type: 'file' | 'directory'): Promise<FileInfo | null> {
	const file: FileInfo = { name, type, path: childPath(currentDirectory.value.path, name) }
	try {
		if (type === 'directory') {
			await invoke('plugin:files|file_create_directory', {
				instanceId: instanceId.value,
				path: toRelativePath(file.path),
			})
		} else {
			await writeBytes(file.path, new Uint8Array(), true)
		}
		await refresh()
		return file
	} catch (e) {
		notifyError(formatMessage(commonMessages.createFailedLabel), e)
		return null
	}
}

async function renameFile(file: FileInfo, destination: string, failedLabel: string) {
	try {
		await invoke('plugin:files|file_rename', {
			instanceId: instanceId.value,
			source: toRelativePath(file.path),
			destination: toRelativePath(destination),
		})
		await refresh()
		return infoFrom({ name: file.name, type: file.type, path: destination })
	} catch (e) {
		notifyError(failedLabel, e)
		return null
	}
}

async function renameItem(file: FileInfo, newName: string): Promise<FileInfo | null> {
	return renameFile(
		file,
		childPath(parentInfoFrom(file).path, newName),
		formatMessage(commonMessages.renameFailedLabel),
	)
}

async function moveItem(file: FileInfo, destination: string): Promise<FileInfo | null> {
	return renameFile(file, destination, formatMessage(commonMessages.moveFailedLabel))
}

async function deleteItem(file: FileInfo, recursive: boolean): Promise<boolean> {
	try {
		await invoke('plugin:files|file_delete', {
			instanceId: instanceId.value,
			path: toRelativePath(file.path),
			recursive,
		})
		await refresh()
		return true
	} catch (e) {
		notifyError(formatMessage(commonMessages.deleteFailedLabel), e)
		return false
	}
}

async function writeFile(file: FileInfo, content: ArrayBuffer) {
	if (file.type !== 'file') return
	await writeBytes(file.path, new Uint8Array(content))
	await queryClient.invalidateQueries({
		queryKey: instanceKeys.fileContent(instanceId.value, file.path),
	})
}

async function downloadFile(file: FileInfo) {
	await invoke('plugin:files|file_save_as', {
		instanceId: instanceId.value,
		filePath: toRelativePath(file.path),
	})
}

const uploadState = ref<UploadState>({
	isUploading: false,
	currentFileName: null,
	currentFileProgress: 0,
	uploadedBytes: 0,
	totalBytes: 0,
	completedFiles: 0,
	totalFiles: 0,
})

async function uploadFiles(files: File[]) {
	if (files.length === 0) return

	const directory = currentDirectory.value.path
	uploadState.value = {
		isUploading: true,
		currentFileName: '',
		currentFileProgress: 0,
		uploadedBytes: 0,
		totalBytes: files.reduce((sum, f) => sum + f.size, 0),
		completedFiles: 0,
		totalFiles: files.length,
	}
	try {
		for (const file of files) {
			uploadState.value.currentFileName = file.name
			const buffer = await file.arrayBuffer()
			await writeBytes(childPath(directory, file.name), new Uint8Array(buffer))
			uploadState.value.completedFiles++
			uploadState.value.uploadedBytes += file.size
			uploadState.value.currentFileProgress = 1
		}
	} catch (e) {
		notifyError(formatMessage(commonMessages.uploadFailedLabel), e)
	} finally {
		uploadState.value.isUploading = false
		await refresh()
	}
}

async function extractFile(path: string, override: boolean, dry: boolean) {
	try {
		return await invoke<ExtractDryRunResult | undefined>('plugin:files|file_extract_zip', {
			instanceId: instanceId.value,
			filePath: toRelativePath(path),
			overrideConflicts: override,
			dryRun: dry,
		})
	} catch (e) {
		notifyError(formatMessage(commonMessages.extractFailedLabel), e)
	}
}

useAppEvent('instance', async (event) => {
	debug('app event: instance =', event.event, 'path =', event.instance_id)
	if (event.instance_id === instanceId.value && event.event === 'synced') {
		debug('app event: synced instance matched, calling refresh')
		await refresh()
	}
})

/**
 * Only the first directory load gates the whole page. Later directory loads are shown by the
 * files layout itself, so its tabs and sidebar stay in place while a directory loads.
 */
const initialLoadPending = ref(true)
watch(
	() => directoryTree.get(currentDirectory.value).isLoading.value,
	(loading) => {
		if (!loading) initialLoadPending.value = false
	},
)

watch(instanceId, async () => {
	debug('watch instance.id: changed to', instanceId.value)
	initialLoadPending.value = true
	currentLocation.value = infoFrom('/')
	expandedDirectories.value = []
	await instanceRootQuery.refetch()
	await refresh()
})

await instanceRootQuery.suspense()
await queryClient.ensureQueryData(directoryQueryOptions('/')).catch(() => undefined)
initialLoadPending.value = false

provideFileManager({
	workspaceId: computed(() => (instanceId.value ? `instance:${instanceId.value}` : null)),
	isReadOnly,
	readOnlyReason: computed(() => formatMessage(messages.readOnly)),
	directoryTree,
	loading: computed(() => directoryTree.get(currentDirectory.value).isLoading.value),
	error: computed(() => directoryTree.get(currentDirectory.value).loadError.value ?? null),
	currentFile: computed(() => currentLocation.value),
	currentDirectory,
	navigateTo,
	createItem,
	renameItem,
	moveItem,
	deleteItem,
	writeFile,
	downloadFile,
	uploadFiles,
	uploadState,
	extractFile,
	refresh,
	isRefreshing,
	basePath: instanceRoot,
	openInFolder: (path: string) => highlightInFolder(path),
	downloadButtonLabel: formatMessage(messages.saveAs),
	uploadingLabel: (completed: number, total: number) =>
		formatMessage(messages.addingFiles, { completed, total }),
})
</script>

<template>
	<ReadyTransition :pending="initialLoadPending">
		<div class="[--files-viewport-height:calc(100vh_-_var(--top-bar-height))]">
			<FilePageLayout :show-refresh-button="true" />
		</div>
	</ReadyTransition>
</template>
