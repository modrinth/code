<script setup lang="ts">
import type { Kyros } from '@modrinth/api-client'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import {
	computed,
	type ComputedRef,
	effectScope,
	markRaw,
	onScopeDispose,
	type Ref,
	ref,
	watch,
} from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ReadyTransition from '#ui/components/base/ReadyTransition.vue'
import { useReadyState } from '#ui/composables'
import { useUploadSessionUpload } from '#ui/composables/hosting/kyros-session-upload'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import { childPath, normalizeDirectoryPath, parentInfoFrom } from '#ui/layouts'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

import FilePageLayout from '../../../shared/files-tab/layout.vue'
import {
	type DirectoryResult,
	type DirectoryTree,
	type FileInfo,
	type FileItemResult,
	type FileItemResultFrom,
	type FileQueryResult,
	type FileResult,
	type FileTypes,
	provideFileManager,
} from '../../../shared/files-tab/providers/file-manager'
import type { FileItem } from '../../../shared/files-tab/types'
import { infoFromQuery, type QueryableFileTypes, queryFilterFor, queryKeyFor } from './utils'

const props = defineProps<{
	showDebugInfo?: boolean
	showRefreshButton?: boolean
	constrainWidth?: boolean
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

const messages = defineMessages({
	zipCreated: {
		id: 'servers.files.zip-created',
		defaultMessage: 'ZIP created',
	},
	zipCreatedDescription: {
		id: 'servers.files.zip-created-description',
		defaultMessage: 'Created {destination}',
	},
})

const zippingFolder = ref(false)

const serverBusy = computed(() => busyReasons.value.length > 0)
const busyTooltip = computed(() =>
	busyReasons.value.length > 0 ? formatMessage(busyReasons.value[0].reason) : undefined,
)
const fileWriteDisabled = computed(
	() => !canWriteFiles.value || serverBusy.value || zippingFolder.value,
)
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
const currentLocation = ref<FileInfo>(infoFromQuery(route))
const currentDirectory = computed<FileInfo<'directory'>>(() => {
	const location = currentLocation.value
	return location.type != 'file'
		? (location as FileInfo<'directory'>)
		: parentInfoFrom(location.path)
})

/**
 * The URL holds the location's own path, with `editing` marking it as a file, so that
 * `infoFromQuery` reads back exactly the location that was navigated to.
 */
function navigateTo(file: FileInfo) {
	const { editing: _, ...query } = route.query
	router.push({
		query: file.type == 'file' ? { ...query, path: file.path, editing: 'true' } : { ...query, path: file.path },
	})

	currentLocation.value = file
}

// TODO: MAYBE RESTRICT TO ONLY FILE EDITING?
// Sync editing state from URL
watch(
	() => route.query,
	(newQuery, oldQuery) => {
		const newInfo = infoFromQuery(newQuery)
		const oldInfo = infoFromQuery(oldQuery)
		if (newInfo.path != oldInfo.path) {
			currentLocation.value = newInfo
		}
	},
	{ deep: true },
)

function isVisibleFileItem(item: Kyros.Files.v0.DirectoryItem) {
	return !item.path.split('/').includes('.modrinth-staged')
}

type FileCache = { hash: string; buffer: ArrayBuffer }

const fileQueryOptions: QueryOptions<'file', FileCache> = {
	queryFn: async (file) => {
		const buffer = await (await client.kyros.files_v0.downloadFile(file.path)).arrayBuffer()

		const hash = await sha512(buffer)

		return { hash: hash, buffer: markRaw(buffer) } as FileCache
	},
	structuralSharing: (oldData, newData) => {
		if (!oldData) return newData
		return oldData.hash === newData.hash ? oldData : newData
	},
}

const directoryQueryOptions: QueryOptions<'directory', Kyros.Files.v0.DirectoryResponse> = {
	queryFn: (file) => client.kyros.files_v0.listDirectory(file.path, 1, 2000),
}

function queryFileEntry(file: FileInfo<'file'>): FileResult & FileQueryResult {
	return queryFile(
		file,
		(queryData) => computed(() => queryData.value?.buffer ?? null),
		fileQueryOptions,
	)
}

// TODO: PAGE SIZE IS LIKE 2000 BUT IN CASES WHERE SOMEONE SOME HOW HAS MORE WE MAY WANT TO
// LAZY LOAD THE REST OR SOMETHING?
function queryDirectoryEntries(file: FileInfo<'directory'>): DirectoryResult & FileQueryResult {
	return queryFile(
		file,
		(queryData) => {
			return computed<FileItem[]>(() => (queryData.value?.items ?? [])
				.filter(isVisibleFileItem)
				.map((item) => { return {...item, path: normalizeDirectoryPath(item.path)} }))
		},
		directoryQueryOptions,
	)
}

type QueryOptions<T extends QueryableFileTypes, Q> = {
	queryFn: (file: FileInfo<T>) => Promise<Q>
	structuralSharing?: (oldData: Partial<Q> | undefined, newData: Partial<Q>) => Partial<Q>
}

function queryFile<T extends QueryableFileTypes, Q, R>(
	fileInfo: FileInfo<T>,
	func: (queryData: Ref<Q | undefined>) => ComputedRef<R>,
	{ queryFn, structuralSharing }: QueryOptions<T, Q>,
): FileItemResult<R, T> & FileQueryResult {
	const key = queryKeyFor(serverId, undefined, fileInfo)

	const {
		data,
		isLoading,
		error: loadError,
	} = useQuery(
		{
			queryKey: key as readonly string[],
			queryFn: () => queryFn(fileInfo),
			structuralSharing: structuralSharing as (
				oldData: unknown | undefined,
				newData: unknown,
			) => unknown,
			staleTime: 30_000,
		},
		queryClient,
	)

	return {
		...fileInfo,
		data: func(data),
		isLoading: isLoading,
		filesReadyPending: useReadyState({ isLoading, data }),
		loadError: loadError,
	}
}

async function sha512(buffer: ArrayBuffer) {
	const hashBuffer = await crypto.subtle.digest('SHA-512', buffer)
	return Array.from(new Uint8Array(hashBuffer))
		.map((b) => b.toString(16).padStart(2, '0'))
		.join('')
}

function prefetchDirectory(file: FileInfo<'directory'>) {
	prefetchFile(file, directoryQueryOptions)
}

function prefetchFileEntry(file: FileInfo<'file'>) {
	prefetchFile(file, fileQueryOptions)
}

function prefetchFile<T extends QueryableFileTypes, Q>(
	file: FileInfo<T>,
	{ queryFn, structuralSharing }: QueryOptions<T, Q>,
): void {
	const key = queryKeyFor(serverId, undefined, file)
	queryClient.prefetchQuery({
		queryKey: key as readonly string[],
		queryFn: () => queryFn(file),
		structuralSharing: structuralSharing as (
			oldData: unknown | undefined,
			newData: unknown,
		) => unknown,
		staleTime: 30_000,
	})
}

function getQueryKey() {
	return ['files', serverId, normalizeDirectoryPath(currentDirectory.value.path)]
}

const isRefreshing = ref<boolean>(false)

async function refreshList() {
	isRefreshing.value = true
	await queryClient.invalidateQueries({ queryKey: ['files', serverId] })
	isRefreshing.value = false
}

// Mutations
const deleteMutation = useMutation({
	mutationFn: async ({ file, recursive }: { file: FileInfo; recursive: boolean }) =>
		client.kyros.files_v0.deleteFileOrFolder(file.path, recursive),
	onMutate: async ({ file }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)
		queryClient.setQueryData(queryKey, (old: Kyros.Files.v0.DirectoryResponse | undefined) => {
			if (!old) return old
			return { ...old, items: old.items.filter((item) => item.path !== file.path) }
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
	mutationFn: async ({ file, newName }: { file: FileInfo; newName: string }) => {
		await client.kyros.files_v0.renameFileOrFolder(file.path, newName)
		const parts = file.path.split('/')
		return {
			type: file.type,
			name: newName,
			path: normalizeDirectoryPath(
				parts.length > 0 ? [...parts.slice(0, -1), newName].join('/') : newName,
			),
		}
	},
	onMutate: async ({ file, newName }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)
		queryClient.setQueryData(queryKey, (old: Kyros.Files.v0.DirectoryResponse | undefined) => {
			if (!old) return old
			return {
				...old,
				items: old.items.map((item) =>
					item.path === file.path
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
	mutationFn: async ({ source, destination }: { source: FileInfo; destination: string }) => {
		await client.kyros.files_v0.moveFileOrFolder(source.path, destination)
		return {
			name: source.name,
			type: source.type,
			path: normalizeDirectoryPath(destination),
		}
	},
	onMutate: async ({ source }) => {
		const queryKey = getQueryKey()
		await queryClient.cancelQueries({ queryKey })
		const previous = queryClient.getQueryData(queryKey)
		queryClient.setQueryData(queryKey, (old: Kyros.Files.v0.DirectoryResponse | undefined) => {
			if (!old) return old
			return { ...old, items: old.items.filter((item) => item.path !== source.path) }
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
	if (fileWriteDisabled.value) return
	const target = path.replace(/\.zip$/i, '')
	if (dry) {
		return await client.kyros.files_v0.extractFile(path, override, true, target)
	}
	await client.kyros.files_v0.extractFile(path, override, false, target)
}

async function writeFile(file: FileInfo, buffer: ArrayBuffer): Promise<void> {
	if (fileWriteDisabled.value || file.type != 'file') return
	await client.kyros.files_v0.updateFile(file.path, new Blob([buffer]))
	await queryClient.invalidateQueries({ queryKey: queryFilterFor(serverId, undefined, 'file') })
	// TODO: POKE CAL ABOUT THIS
	//queryClient.invalidateQueries({queryKey: ['servers', 'detail', serverId]})
}

async function downloadFile(file: FileInfo): Promise<void> {
	try {
		const fileData = await client.kyros.files_v0.downloadFile(file.path)
		if (fileData) {
			saveBlob(fileData, file.name ?? 'modrinth-file-download')
		}
	} catch {
		addNotification({
			title: formatMessage(commonMessages.downloadFailedLabel),
			text: 'Could not download the file.',
			type: 'error',
		})
	}
}

function saveBlob(blob: Blob, fileName: string) {
	const link = document.createElement('a')
	link.href = window.URL.createObjectURL(blob)
	link.download = fileName
	link.click()
	window.URL.revokeObjectURL(link.href)
}

async function statFile(path: string): Promise<Kyros.Files.v1.FileStatResponse> {
	if (!worldId.value) throw new Error('No active world')
	return client.kyros.files_v1.stat(worldId.value, { path })
}

async function createZip(data: Kyros.Files.v1.ZipRequest, destination?: string): Promise<void> {
	if (!worldId.value || fileWriteDisabled.value) return
	const operationId = `local-zip-${crypto.randomUUID()}`
	const source =
		data.target_type === 'Directory'
			? `${data.path.split('/').pop() || data.path}.zip`
			: data.target
	const updateOperation = (record: Partial<Kyros.Files.v1.ZipProgress>, state = 'ongoing') => {
		serverContext.upsertLocalFileOperation?.({
			id: operationId,
			op: 'zip',
			src: source,
			state,
			progress: (record.progress ?? 0) / 100,
			cancellable: false,
			error: record.error,
		})
	}
	zippingFolder.value = true
	updateOperation({ progress: 0 })
	try {
		await client.kyros.files_v1.createZip(worldId.value, data, (record) => {
			updateOperation(record, record.done ? 'done' : 'ongoing')
		})
		updateOperation({ progress: 100 }, 'done')
		addNotification({
			title: formatMessage(messages.zipCreated),
			text: destination
				? formatMessage(messages.zipCreatedDescription, { destination })
				: undefined,
			type: 'success',
		})
		await refreshList()
	} catch (error) {
		updateOperation(
			{ progress: 0, error: error instanceof Error ? error.message : undefined },
			'failure-error',
		)
	} finally {
		zippingFolder.value = false
	}
}

async function zipFolder(file: FileInfo<'directory'>): Promise<void> {
	await createZip({ target_type: 'Directory', path: file.path })
}

async function zipPaths(
	files: FileInfo[],
	targetDirectory: FileInfo<'directory'>,
	archiveName: string,
): Promise<void> {
	const targetPath = targetDirectory.path
	await createZip(
		{
			target_type: 'ManyPaths',
			parent: targetPath,
			include: files.map((file) => file.path),
			target: archiveName,
		},
		`${targetPath}/${archiveName}`.replace('//', '/'),
	)
}

watch(
	() => fsOps.value,
	() => {
		refreshList()
	},
)

// Restart
async function restartServer() {
	if (!canUsePowerActions.value) return
	await client.archon.servers_v0.power(serverId, 'Restart')
}

function getSessionUploadFilename(fileName: string) {
	const basePath = currentDirectory.value.path.split('/').filter(Boolean).join('/')
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
		if (result === 'completed') await refreshList()
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

const fileQueries = new Map<string, FileItemResultFrom<'file' | 'directory'> & FileQueryResult>()

/** Files and directories are cached apart, so a lookup can never return the other kind's result. */
function queryCacheKey(info: FileInfo) {
	return `${info.type == 'file' ? 'file' : 'directory'}:${normalizeDirectoryPath(info.path)}`
}
const expandedDirectories: Ref<string[]> = ref([])

/** Owns the lazily created directory queries so they're disposed with this page, wherever they were first requested from. */
const fileSystemScope = effectScope()
onScopeDispose(() => fileSystemScope.stop())

const directoryTree = {
	prefetch: <T extends FileTypes>(info: FileInfo<T>) => {
		if (fileQueries.has(queryCacheKey(info))) return
		if (info.type == 'file') {
			prefetchFileEntry(info as FileInfo<'file'>)
		} else {
			prefetchDirectory(info as FileInfo<'directory'>)
		}
	},
	get: <T extends FileTypes>(info: FileInfo<T>): FileItemResultFrom<T> & FileQueryResult => {
		const key = queryCacheKey(info)
		let query = fileQueries.get(key)
		if (query == null) {
			query = fileSystemScope.run(() =>
				info.type == 'file'
					? queryFileEntry(info as FileInfo<'file'>)
					: queryDirectoryEntries(info as FileInfo<'directory'>),
			)!
			fileQueries.set(key, query)
		}
		return query as FileItemResultFrom<T> & FileQueryResult
	},
	expandedEntries: expandedDirectories,
} satisfies DirectoryTree

// Provide the file manager context
provideFileManager({
	workspaceId: `server:${serverId}`,
	directoryTree,
	loading: computed(() => directoryTree.get(currentDirectory.value).isLoading.value),
	error: computed(() => directoryTree.get(currentDirectory.value).loadError.value ?? null),
	currentFile: computed(() => currentLocation.value),
	currentDirectory,
	navigateTo,
	createItem: async (name, type) => {
		if (fileWriteDisabled.value) return null
		const path = childPath(currentDirectory.value.path, name)
		await createMutation.mutateAsync({ path, type })
		return {
			name: name,
			path: path,
			type: type,
		} as FileInfo
	},
	renameItem: async (file, newName) => {
		if (fileWriteDisabled.value) return null
		return await renameMutation.mutateAsync({ file, newName }).catch(() => null)
	},
	moveItem: async (source, destination) => {
		if (fileWriteDisabled.value) return null
		return await moveMutation.mutateAsync({ source, destination }).catch(() => null)
	},
	deleteItem: async (file, recursive) => {
		if (fileWriteDisabled.value) return false
		return await deleteMutation
			.mutateAsync({ file, recursive })
			.then(() => true)
			.catch(() => false)
	},
	writeFile,
	downloadFile,
	statFile,
	zipFolder,
	zipPaths,
	uploadFiles,
	cancelUpload,
	uploadState,
	refresh: refreshList,
	isRefreshing,
	isBusy: fileWriteDisabled,
	busyTooltip: fileWriteDisabledTooltip,
	busyWarning,
	extractFile,
	showInstallFromUrl: true,
	canRestart: canUsePowerActions.value,
	restartServer,
	canShareToMclogs: true,
})

/**
 * Only the first directory load gates the whole page. Later directory loads are shown by the
 * files layout itself, so its tabs and sidebar stay in place while a directory loads.
 */
const initialLoadPending = ref(true)
watch(
	() => directoryTree.get(currentDirectory.value).filesReadyPending.value,
	(pending) => {
		if (!pending) initialLoadPending.value = false
	},
	{ immediate: true },
)
</script>

<template>
	<ReadyTransition :pending="initialLoadPending">
		<FilePageLayout
			:show-debug-info="props.showDebugInfo"
			:show-refresh-button="props.showRefreshButton"
			:constrain-width="constrainWidth"
		/>
	</ReadyTransition>
</template>
