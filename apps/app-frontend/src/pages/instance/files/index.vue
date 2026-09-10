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
	readOnly: {
		id: 'instance.files.managed-content-read-only',
		defaultMessage: 'Managed content is read-only here. Add, disable, update, or remove it from the Content tab.',
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
const items = ref<FileItem[]>([])
/** True until the first directory read for the current instance path finishes (initial load only). */
const firstPaintPending = ref(true)
const loading = ref(true)
const error = ref<Error | null>(null)
const currentPath = ref('')
const editingFile = ref<EditingFile | null>(null)

debug('setup: start, instance.id =', instanceId.value)

async function listDirectory(dirPath: string): Promise<FileItem[]> {
	return invoke('plugin:files|file_list', { instanceId: instanceId.value, path: dirPath })
}

function isReadOnly(path: string): boolean {
	const normalized = path.startsWith('/') ? path.slice(1) : path
	return normalized.split('/')[0].toLowerCase() === 'mods'
		|| items.value.some((item) => item.path === normalized && item.readOnly === true)
}

async function writeBytes(path: string, bytes: Uint8Array, createOnly = false) {
	await invoke('plugin:files|file_write', {
		instanceId: instanceId.value, path, bytes: Array.from(bytes), createOnly,
	})
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
	try {
		if (type === 'directory') {
			await invoke('plugin:files|file_create_directory', { instanceId: instanceId.value, path: targetPath })
		} else {
			await writeBytes(targetPath, new Uint8Array(), true)
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
	const parentDir = path.includes('/') ? path.substring(0, path.lastIndexOf('/')) : ''
	const newPath = parentDir ? `${parentDir}/${newName}` : newName
	try {
		await invoke('plugin:files|file_rename', { instanceId: instanceId.value, source: path, destination: newPath })
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
		await invoke('plugin:files|file_rename', { instanceId: instanceId.value, source, destination })
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
		await invoke('plugin:files|file_delete', { instanceId: instanceId.value, path, recursive })
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
	const bytes = await invoke<number[]>('plugin:files|file_read', { instanceId: instanceId.value, path })
	return new TextDecoder().decode(new Uint8Array(bytes))
}

async function handleReadFileAsBlob(path: string): Promise<Blob> {
	const bytes = await invoke<number[]>('plugin:files|file_read', { instanceId: instanceId.value, path })
	return new Blob([new Uint8Array(bytes)])
}

async function handleWriteFile(path: string, content: string) {
	await writeBytes(path, new TextEncoder().encode(content))
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
			const targetPath = currentPath.value ? `${currentPath.value}/${file.name}` : file.name
			await writeBytes(targetPath, new Uint8Array(buffer))
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
	isReadOnly,
	readOnlyReason: computed(() => formatMessage(messages.readOnly)),
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
		<div>
		<p v-if="isReadOnly(currentPath)" class="m-0 mb-4 text-sm text-secondary">
			{{ formatMessage(messages.readOnly) }}
		</p>
		<FilePageLayout :show-refresh-button="true" />
		</div>
	</ReadyTransition>
</template>
