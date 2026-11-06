<script setup lang="ts">
import type { SharedInstances } from '@modrinth/api-client'
import { LoaderCircleIcon } from '@modrinth/assets'
import {
	ConfirmLeaveModal,
	type EditingFile,
	type FileItem,
	FilePageLayout,
	injectNotificationManager,
	provideFileManager,
} from '@modrinth/ui'
import { computed, ref, toRef, watch } from 'vue'

import { useSharedInstanceModeration } from '~/composables/shared-instance-moderation'

type ExternalFile = SharedInstances.Instances.v1.ExternalFile

const props = defineProps<{
	instance: { id: string; name: string }
	version: number
	files: ExternalFile[]
	loading: boolean
	error: Error | null
}>()

const emit = defineEmits<{
	refresh: []
}>()

const { addNotification } = injectNotificationManager()
const { pending, deleteFiles } = useSharedInstanceModeration()

const scroller = ref<HTMLElement | null>(null)
const confirmDeleteModal = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)
const pendingDeleteCount = ref(0)
const currentPath = ref('/')
const editingFile = ref<EditingFile | null>(null)

function normalizePath(path: string) {
	return path
		.split(/[\\/]+/)
		.filter(Boolean)
		.join('/')
}

const filesByPath = computed(
	() => new Map(props.files.map((file) => [normalizePath(file.file_name), file])),
)

const items = computed<FileItem[]>(() => {
	const directory = normalizePath(currentPath.value)
	const prefix = directory ? `${directory}/` : ''
	const files: FileItem[] = []
	const directories = new Map<string, Set<string>>()

	for (const [path, file] of filesByPath.value) {
		if (!path.startsWith(prefix)) continue
		const [name, child] = path.slice(prefix.length).split('/')
		if (child === undefined) {
			files.push({
				name,
				type: 'file',
				path: `/${path}`,
				size: file.file_size ?? null,
				modified: 0,
				created: 0,
			})
		} else {
			directories.set(name, (directories.get(name) ?? new Set()).add(child))
		}
	}

	return [
		...[...directories].map(
			([name, children]): FileItem => ({
				name,
				type: 'directory',
				path: `/${prefix}${name}`,
				count: children.size,
				modified: 0,
				created: 0,
			}),
		),
		...files,
	]
})

function filesAt(path: string) {
	const target = normalizePath(path)
	return [...filesByPath.value]
		.filter(([filePath]) => filePath === target || filePath.startsWith(`${target}/`))
		.map(([, file]) => file)
}

function removeFiles(files: ExternalFile[]) {
	return deleteFiles(
		props.instance,
		props.version,
		files.map((file) => ({ fileName: file.file_name, fileType: file.file_type })),
	)
}

async function deleteItem(path: string) {
	await removeFiles(filesAt(path))
}

async function deleteItems(selected: FileItem[]) {
	const files = selected.flatMap((item) => filesAt(item.path))
	pendingDeleteCount.value = files.length
	if (!(await confirmDeleteModal.value?.prompt())) return false
	return removeFiles(files)
}

async function readFileAsBlob(path: string) {
	const url = filesByPath.value.get(normalizePath(path))?.url
	if (!url) throw new Error(`No download URL for ${path}`)

	const response = await fetch(url)
	if (!response.ok) throw new Error(`Failed to download ${path}: ${response.status}`)
	return response.blob()
}

async function readFile(path: string) {
	return (await readFileAsBlob(path)).text()
}

async function downloadFile(path: string, fileName: string) {
	const url = filesByPath.value.get(normalizePath(path))?.url
	if (!url) {
		addNotification({
			type: 'error',
			title: 'Failed to download file',
			text: `${fileName} has no download URL.`,
		})
		return
	}
	window.open(url, '_blank', 'noopener,noreferrer')
}

async function unsupported() {}

provideFileManager({
	items,
	loading: toRef(props, 'loading'),
	error: toRef(props, 'error'),
	currentPath,
	navigateTo: (path) => {
		currentPath.value = path
	},
	editingFile,
	startEditing: (file) => {
		editingFile.value = file
	},
	stopEditing: () => {
		editingFile.value = null
	},
	createItem: unsupported,
	renameItem: unsupported,
	moveItem: unsupported,
	writeFile: unsupported,
	uploadFiles: unsupported,
	deleteItem,
	deleteItems,
	readFile,
	readFileAsBlob,
	downloadFile,
	refresh: () => emit('refresh'),
	isBusy: pending,
	busyTooltip: computed(() => (pending.value ? 'Deleting files…' : undefined)),
	browseOnly: true,
	hideTimestamps: true,
})

watch([currentPath, editingFile], () => {
	if (scroller.value) scroller.value.scrollTop = 0
})
</script>

<template>
	<div ref="scroller" class="overflow-y-auto px-6 pb-6 pt-4">
		<div
			v-if="loading && !files.length"
			class="flex items-center justify-center gap-2 p-8 text-secondary"
		>
			<LoaderCircleIcon class="size-6 animate-spin" />
			Loading files…
		</div>
		<FilePageLayout inline-action-bar />
		<ConfirmLeaveModal
			ref="confirmDeleteModal"
			title="Delete files?"
			:header="`Delete ${pendingDeleteCount} ${pendingDeleteCount === 1 ? 'file' : 'files'}`"
			body="The selected files are removed from this version of the shared instance. Members who install it afterwards won't receive them."
			stay-label="Cancel"
			leave-label="Delete files"
			admonition-type="critical"
		/>
	</div>
</template>
