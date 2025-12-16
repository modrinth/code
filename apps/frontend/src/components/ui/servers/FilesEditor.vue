<template>
	<div class="flex h-full w-full flex-col">
		<FilesRenameItemModal ref="renameModal" :item="file" @rename="handleRenameItem" />

		<FilesEditingNavbar
			:file-name="file?.name"
			:is-image="isEditingImage"
			:file-path="file?.path"
			:breadcrumb-segments="breadcrumbSegments"
			@cancel="handleCancel"
			@save="() => saveFileContent(true)"
			@save-as="saveFileContentAs"
			@save-restart="saveFileContentRestart"
			@share="requestShareLink"
			@navigate="(index) => emit('navigate', index)"
		/>

		<div class="h-full w-full flex-grow">
			<CodeEditor
				v-if="!isEditingImage"
				v-model:value="fileContent"
				:language="editorLanguage"
				theme="vs-dark"
				:options="editorOptions"
				height="750px"
				class="overflow-hidden rounded-b-lg"
				@editor-did-mount="onEditorInit"
			/>
			<FilesImageViewer v-else-if="isEditingImage && imagePreview" :image-blob="imagePreview" />
			<div
				v-else-if="isLoading"
				class="flex h-[750px] items-center justify-center rounded-b-lg bg-bg-raised"
			>
				<SpinnerIcon class="h-8 w-8 animate-spin text-secondary" />
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import {
	getEditorLanguage,
	getFileExtension,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	isImageFile,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import * as monaco from 'monaco-editor'
import { CodeEditor } from 'monaco-editor-vue3'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import FilesEditingNavbar from '~/components/ui/servers/FilesEditingNavbar.vue'
import FilesImageViewer from '~/components/ui/servers/FilesImageViewer.vue'
import FilesRenameItemModal from '~/components/ui/servers/FilesRenameItemModal.vue'

const props = defineProps<{
	file: { name: string; type: string; path: string } | null
	breadcrumbSegments: string[]
}>()

const emit = defineEmits<{
	close: []
	navigate: [index: number]
}>()

const notifications = injectNotificationManager()
const { addNotification } = notifications
const client = injectModrinthClient()
const serverContext = injectModrinthServerContext()
const { serverId } = serverContext
const queryClient = useQueryClient()

const modulesLoaded = inject<Promise<void>>('modulesLoaded')

// Internal state
const fileContent = ref('')
const isEditingImage = ref(false)
const imagePreview = ref<Blob | null>(null)
const isLoading = ref(false)
const renameModal = ref()
const closeAfterRename = ref(false)
const editorInstance = ref<any>(null)

// Monaco editor options
const editorOptions = {
	fontSize: 16,
	minimap: { enabled: false },
	automaticLayout: true,
	scrollBeyondLastLine: false,
	wordWrap: 'on' as const,
	tabSize: 4,
}

const editorLanguage = computed(() => {
	const ext = getFileExtension(props.file?.name ?? '')
	return getEditorLanguage(ext)
})

// Load file content when file prop changes
watch(
	() => props.file,
	async (newFile) => {
		if (newFile) {
			await loadFileContent(newFile)
		} else {
			resetState()
		}
	},
	{ immediate: true },
)

async function loadFileContent(file: { name: string; type: string; path: string }) {
	isLoading.value = true
	try {
		window.scrollTo(0, 0)
		const extension = getFileExtension(file.name)

		if (file.type === 'file' && isImageFile(extension)) {
			// Images are not prefetched, fetch directly
			const content = await client.kyros.files_v0.downloadFile(file.path)
			isEditingImage.value = true
			imagePreview.value = content
		} else {
			isEditingImage.value = false
			// Check cache first for text files (may have been prefetched on hover)
			const cachedContent = queryClient.getQueryData<string>(['file-content', serverId, file.path])
			if (cachedContent) {
				fileContent.value = cachedContent
			} else {
				const content = await client.kyros.files_v0.downloadFile(file.path)
				fileContent.value = await content.text()
			}
		}
	} catch (error) {
		console.error('Error fetching file content:', error)
		addNotification({
			title: 'Failed to open file',
			text: 'Could not load file contents.',
			type: 'error',
		})
		emit('close')
	} finally {
		isLoading.value = false
	}
}

function resetState() {
	fileContent.value = ''
	isEditingImage.value = false
	imagePreview.value = null
	closeAfterRename.value = false
}

function onEditorInit(editor: any) {
	editorInstance.value = editor
	// Add Ctrl/Cmd+S save shortcut
	editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
		saveFileContent(false)
	})
}

async function saveFileContent(exit: boolean = true) {
	if (!props.file) return

	try {
		await client.kyros.files_v0.updateFile(props.file.path, fileContent.value)

		if (exit) {
			await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
			emit('close')
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
	await saveFileContent(false)
	await client.archon.servers_v0.power(serverId, 'Restart')

	addNotification({
		title: 'Server restarted',
		text: 'Your server has been restarted.',
		type: 'success',
	})

	emit('close')
}

async function saveFileContentAs() {
	await saveFileContent(false)
	closeAfterRename.value = true
	renameModal.value?.show(props.file)
}

async function handleRenameItem(newName: string) {
	if (!props.file) return

	try {
		await client.kyros.files_v0.renameFileOrFolder(props.file.path, newName)

		addNotification({ title: 'Renamed', text: `Renamed to ${newName}`, type: 'success' })

		if (closeAfterRename.value) {
			await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
			closeAfterRename.value = false
			emit('close')
		}
	} catch (err: any) {
		addNotification({ title: 'Rename failed', text: err.message, type: 'error' })
	}
}

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
		addNotification({
			title: 'Failed to share file',
			text: 'Could not upload to mclo.gs.',
			type: 'error',
		})
	}
}

function handleCancel() {
	resetState()
	emit('close')
}

onMounted(async () => {
	await modulesLoaded
})

onUnmounted(() => {
	editorInstance.value = null
	resetState()
})
</script>
