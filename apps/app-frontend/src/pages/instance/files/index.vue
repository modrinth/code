<script setup lang="ts">
import type { EditingFile, FileItem, UploadState } from '@modrinth/ui'
import {
	commonMessages,
	defineMessages,
	FilePageLayout,
	injectNotificationManager,
	provideFileManager,
	ReadyTransition,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { invoke } from '@tauri-apps/api/core'
import {
	mkdir,
	readDir,
	readFile as readFileBytes,
	readTextFile,
	remove,
	rename,
	stat,
	writeFile as writeFileBytes,
	writeTextFile,
} from '@tauri-apps/plugin-fs'
import { computed, ref, watch } from 'vue'

import { useAppEvent } from '@/composables/use-app-event'
import { get_full_path } from '@/helpers/instance'
import { highlightInFolder } from '@/helpers/utils'

import { injectInstancePage } from '../instance-context'
import { instanceKeys } from '../query-options'

const instancePage = injectInstancePage()
const instanceId = instancePage.instanceId

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const debug = useDebugLogger('Files')

const messages = defineMessages({
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
const items = ref<FileItem[]>([])
/** True until the first directory read for the current instance path finishes (initial load only). */
const firstPaintPending = ref(true)
const loading = ref(true)
const error = ref<Error | null>(null)
const currentPath = ref('')
const editingFile = ref<EditingFile | null>(null)

debug('setup: start, instance.id =', instanceId.value)

function resolvePath(relativePath: string): string {
	return relativePath ? `${instanceRoot.value}/${relativePath}` : instanceRoot.value
}

async function listDirectory(dirPath: string): Promise<FileItem[]> {
	const absPath = resolvePath(dirPath)
	debug('listDirectory: dirPath =', dirPath, 'absPath =', absPath)
	const entries = await readDir(absPath)
	debug('listDirectory: got', entries.length, 'entries')

	const results = await Promise.all(
		entries.map(async (entry) => {
			const entryAbsPath = `${absPath}/${entry.name}`
			let metadata
			try {
				metadata = await stat(entryAbsPath)
			} catch {
				debug('listDirectory: stat failed for', entry.name, '- skipping')
				return null
			}
			const item: FileItem = {
				name: entry.name,
				type: entry.isDirectory ? 'directory' : 'file',
				path: dirPath ? `${dirPath}/${entry.name}` : entry.name,
				modified: metadata.mtime ? Math.floor(metadata.mtime.getTime() / 1000) : 0,
				created: metadata.birthtime ? Math.floor(metadata.birthtime.getTime() / 1000) : 0,
			}
			if (!entry.isDirectory) {
				item.size = metadata.size
			}
			if (entry.isDirectory) {
				try {
					const children = await readDir(entryAbsPath)
					item.count = children.length
				} catch {
					item.count = 0
				}
			}
			return item
		}),
	)
	return results.filter((item): item is FileItem => item !== null)
}

const directoryQuery = useQuery(
	computed(() => ({
		queryKey: instanceKeys.files(instancePage.instanceId.value, currentPath.value),
		queryFn: () => listDirectory(currentPath.value),
		enabled: !!instanceRoot.value,
		staleTime: 30_000,
	})),
)

watch(
	directoryQuery.data,
	(data) => {
		if (!data) return
		items.value = data
		firstPaintPending.value = false
	},
	{ immediate: true },
)
watch(directoryQuery.isFetching, (fetching) => {
	loading.value = fetching
})
watch(directoryQuery.error, (queryError) => {
	error.value = queryError
	if (queryError) items.value = []
})

await instanceRootQuery.suspense()
await directoryQuery.refetch()
firstPaintPending.value = false

async function refresh() {
	debug('refresh: called, currentPath =', currentPath.value, 'instanceRoot =', instanceRoot.value)
	await directoryQuery.refetch()
}

function navigateTo(path: string) {
	debug('navigateTo:', path)
	currentPath.value = path.startsWith('/') ? path.slice(1) : path
	refresh()
}

function startEditing(file: EditingFile) {
	editingFile.value = file
}

function stopEditing() {
	editingFile.value = null
}

async function handleCreateItem(name: string, type: 'file' | 'directory') {
	const targetPath = currentPath.value ? `${currentPath.value}/${name}` : name
	const absPath = resolvePath(targetPath)
	try {
		if (type === 'directory') {
			await mkdir(absPath)
		} else {
			await writeTextFile(absPath, '')
		}
		await refresh()
	} catch (e) {
		addNotification({
			title: formatMessage(commonMessages.createFailedLabel),
			text: e instanceof Error ? e.message : '',
			type: 'error',
		})
	}
}

async function handleRenameItem(path: string, newName: string) {
	const oldAbs = resolvePath(path)
	const parentDir = path.includes('/') ? path.substring(0, path.lastIndexOf('/')) : ''
	const newPath = parentDir ? `${parentDir}/${newName}` : newName
	const newAbs = resolvePath(newPath)
	try {
		await rename(oldAbs, newAbs)
		await refresh()
	} catch (e) {
		addNotification({
			title: formatMessage(commonMessages.renameFailedLabel),
			text: e instanceof Error ? e.message : '',
			type: 'error',
		})
	}
}

async function handleMoveItem(source: string, destination: string) {
	try {
		await rename(resolvePath(source), resolvePath(destination))
		await refresh()
	} catch (e) {
		addNotification({
			title: formatMessage(commonMessages.moveFailedLabel),
			text: e instanceof Error ? e.message : '',
			type: 'error',
		})
	}
}

async function handleDeleteItem(path: string, recursive: boolean) {
	try {
		await remove(resolvePath(path), { recursive })
		await refresh()
	} catch (e) {
		addNotification({
			title: formatMessage(commonMessages.deleteFailedLabel),
			text: e instanceof Error ? e.message : '',
			type: 'error',
		})
	}
}

async function handleReadFile(path: string): Promise<string> {
	return await readTextFile(resolvePath(path))
}

async function handleReadFileAsBlob(path: string): Promise<Blob> {
	const bytes = await readFileBytes(resolvePath(path))
	return new Blob([bytes])
}

async function handleWriteFile(path: string, content: string) {
	await writeTextFile(resolvePath(path), content)
}

async function handleDownloadFile(path: string, _fileName: string) {
	await invoke('plugin:files|file_save_as', {
		instanceId: instanceId.value,
		filePath: path,
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

async function handleUploadFiles(files: File[]) {
	if (files.length === 0) return

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
			const targetPath = resolvePath(
				currentPath.value ? `${currentPath.value}/${file.name}` : file.name,
			)
			await writeFileBytes(targetPath, new Uint8Array(buffer))
			uploadState.value.completedFiles++
			uploadState.value.uploadedBytes += file.size
			uploadState.value.currentFileProgress = 1
		}
	} catch (e) {
		addNotification({
			title: formatMessage(commonMessages.uploadFailedLabel),
			text: e instanceof Error ? e.message : '',
			type: 'error',
		})
	} finally {
		uploadState.value.isUploading = false
		await refresh()
	}
}

async function handleExtractFile(path: string, override: boolean, dry: boolean) {
	try {
		return await invoke('plugin:files|file_extract_zip', {
			instanceId: instanceId.value,
			filePath: path,
			overrideConflicts: override,
			dryRun: dry,
		})
	} catch (e) {
		addNotification({
			title: formatMessage(commonMessages.extractFailedLabel),
			text: e instanceof Error ? e.message : '',
			type: 'error',
		})
	}
}

useAppEvent('instance', async (event) => {
	debug('app event: instance =', event.event, 'path =', event.instance_id)
	if (event.instance_id === instanceId.value && event.event === 'synced') {
		debug('app event: synced instance matched, calling refresh')
		await refresh()
	}
})

watch(instanceId, async () => {
	debug('watch instance.id: changed to', instanceId.value)
	firstPaintPending.value = true
	currentPath.value = ''
	await instanceRootQuery.refetch()
	await refresh()
})

provideFileManager({
	items,
	loading,
	error,
	currentPath,
	navigateTo,
	editingFile,
	startEditing,
	stopEditing,
	createItem: handleCreateItem,
	renameItem: handleRenameItem,
	moveItem: handleMoveItem,
	deleteItem: handleDeleteItem,
	readFile: handleReadFile,
	readFileAsBlob: handleReadFileAsBlob,
	writeFile: handleWriteFile,
	downloadFile: handleDownloadFile,
	uploadFiles: handleUploadFiles,
	uploadState,
	extractFile: handleExtractFile,
	refresh,
	basePath: instanceRoot,
	openInFolder: (path: string) => highlightInFolder(path),
	downloadButtonLabel: formatMessage(messages.saveAs),
	uploadingLabel: (completed: number, total: number) =>
		formatMessage(messages.addingFiles, { completed, total }),
})
</script>

<template>
	<ReadyTransition :pending="firstPaintPending">
		<FilePageLayout :show-refresh-button="true" />
	</ReadyTransition>
</template>
