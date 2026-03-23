<script setup lang="ts">
import type { EditingFile, FileItem, UploadState } from '@modrinth/ui'
import {
	defineMessages,
	FilePageLayout,
	injectNotificationManager,
	provideFileManager,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
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
import { onMounted, onUnmounted, ref, watch } from 'vue'

import { profile_listener } from '@/helpers/events'
import { get_full_path } from '@/helpers/profile'
import type { GameInstance } from '@/helpers/types'
import { highlightInFolder } from '@/helpers/utils'

const props = defineProps<{
	instance: GameInstance
	options: unknown
	offline: boolean
	playing: boolean
	installed: boolean
	isServerInstance: boolean
}>()

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
	deleteFailed: {
		id: 'instance.files.delete-failed',
		defaultMessage: 'Delete failed',
	},
	renameFailed: {
		id: 'instance.files.rename-failed',
		defaultMessage: 'Rename failed',
	},
	moveFailed: {
		id: 'instance.files.move-failed',
		defaultMessage: 'Move failed',
	},
	createFailed: {
		id: 'instance.files.create-failed',
		defaultMessage: 'Create failed',
	},
	uploadFailed: {
		id: 'instance.files.upload-failed',
		defaultMessage: 'Upload failed',
	},
	extractFailed: {
		id: 'instance.files.extract-failed',
		defaultMessage: 'Extract failed',
	},
})

const instanceRoot = ref('')
const items = ref<FileItem[]>([])
const loading = ref(true)
const error = ref<Error | null>(null)
const currentPath = ref('')
const editingFile = ref<EditingFile | null>(null)

debug('setup: start, instance.path =', props.instance.path)

onMounted(async () => {
	debug('onMounted: fired')
	instanceRoot.value = await get_full_path(props.instance.path)
	debug('onMounted: instanceRoot =', instanceRoot.value)
	await refresh()
	debug('onMounted: refresh complete, items =', items.value.length, 'error =', error.value)
})

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

async function refresh() {
	debug('refresh: called, currentPath =', currentPath.value, 'instanceRoot =', instanceRoot.value)
	loading.value = true
	error.value = null
	try {
		items.value = await listDirectory(currentPath.value)
		debug('refresh: success, items =', items.value.length)
	} catch (e) {
		debug('refresh: error =', e)
		error.value = e instanceof Error ? e : new Error(String(e))
		items.value = []
	} finally {
		loading.value = false
	}
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
			title: formatMessage(messages.createFailed),
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
			title: formatMessage(messages.renameFailed),
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
			title: formatMessage(messages.moveFailed),
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
			title: formatMessage(messages.deleteFailed),
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
		instancePath: props.instance.path,
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
			title: formatMessage(messages.uploadFailed),
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
			instancePath: props.instance.path,
			filePath: path,
			overrideConflicts: override,
			dryRun: dry,
		})
	} catch (e) {
		addNotification({
			title: formatMessage(messages.extractFailed),
			text: e instanceof Error ? e.message : '',
			type: 'error',
		})
	}
}

let unlistenProfiles: (() => void) | undefined

onMounted(async () => {
	debug('setup: registering profile_listener')
	unlistenProfiles = await profile_listener(
		async (event: { event: string; profile_path_id: string }) => {
			debug('profile_listener: event =', event.event, 'path =', event.profile_path_id)
			if (event.profile_path_id === props.instance.path && event.event === 'synced') {
				debug('profile_listener: synced event matched, calling refresh')
				await refresh()
			}
		},
	)
	debug('setup: profile_listener registered')
})

onUnmounted(() => {
	unlistenProfiles?.()
})

watch(
	() => props.instance.path,
	async () => {
		debug('watch instance.path: changed to', props.instance.path)
		instanceRoot.value = await get_full_path(props.instance.path)
		currentPath.value = ''
		await refresh()
	},
)

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
	<FilePageLayout :show-refresh-button="true" />
</template>
