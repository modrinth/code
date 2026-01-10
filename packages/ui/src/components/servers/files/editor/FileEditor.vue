<template>
	<div class="flex h-full w-full flex-col gap-4">
		<div class="flex flex-col overflow-hidden rounded-[20px] shadow-md">
			<div class="h-full w-full flex-grow">
				<component
					:is="props.editorComponent"
					v-if="!isEditingImage && props.editorComponent"
					v-model:value="fileContent"
					:lang="editorLanguage"
					theme="modrinth"
					:print-margin="false"
					style="height: 750px; font-size: 1rem"
					class="ace-modrinth rounded-[20px]"
					@init="onEditorInit"
				/>
				<FileImageViewer v-else-if="isEditingImage && imagePreview" :image-blob="imagePreview" />
				<div
					v-else-if="isLoading || !props.editorComponent"
					class="flex h-[750px] items-center justify-center rounded-[20px] bg-bg-raised"
				>
					<SpinnerIcon class="h-8 w-8 animate-spin text-secondary" />
				</div>
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
import { type Component, computed, inject, onMounted, onUnmounted, ref, watch } from 'vue'

import FileImageViewer from './FileImageViewer.vue'

interface MclogsResponse {
	success: boolean
	url?: string
	error?: string
}

const props = defineProps<{
	file: { name: string; type: string; path: string } | null
	editorComponent: Component | null
}>()

const emit = defineEmits<{
	close: []
}>()

const notifications = injectNotificationManager()
const { addNotification } = notifications
const client = injectModrinthClient()
const serverContext = injectModrinthServerContext()
const { serverId } = serverContext
const queryClient = useQueryClient()

const modulesLoaded = inject<Promise<void>>('modulesLoaded')

const fileContent = ref('')
const isEditingImage = ref(false)
const imagePreview = ref<Blob | null>(null)
const isLoading = ref(false)
const editorInstance = ref<unknown>(null)

const editorLanguage = computed(() => {
	const ext = getFileExtension(props.file?.name ?? '')
	return getEditorLanguage(ext)
})

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
		const normalizedPath = file.path.startsWith('/') ? file.path : `/${file.path}`

		if (file.type === 'file' && isImageFile(extension)) {
			const content = await client.kyros.files_v0.downloadFile(normalizedPath)
			isEditingImage.value = true
			imagePreview.value = content
		} else {
			isEditingImage.value = false
			const cachedContent = queryClient.getQueryData<string>([
				'file-content',
				serverId,
				normalizedPath,
			])
			if (cachedContent) {
				fileContent.value = cachedContent
			} else {
				const content = await client.kyros.files_v0.downloadFile(normalizedPath)
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
}

function onEditorInit(editor: {
	commands: {
		addCommand: (cmd: {
			name: string
			bindKey: { win: string; mac: string }
			exec: () => void
		}) => void
	}
}) {
	editorInstance.value = editor

	editor.commands.addCommand({
		name: 'save',
		bindKey: { win: 'Ctrl-S', mac: 'Command-S' },
		exec: () => saveFileContent(false),
	})
}

async function saveFileContent(exit: boolean = true) {
	if (!props.file) return

	try {
		const normalizedPath = props.file.path.startsWith('/') ? props.file.path : `/${props.file.path}`
		await client.kyros.files_v0.updateFile(normalizedPath, fileContent.value)

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

async function saveAndRestart() {
	await saveFileContent(false)
	await client.archon.servers_v0.power(serverId, 'Restart')

	addNotification({
		title: 'Server restarted',
		text: 'Your server has been restarted.',
		type: 'success',
	})

	emit('close')
}

async function shareToMclogs() {
	try {
		const response = await fetch('https://api.mclo.gs/1/log', {
			method: 'POST',
			headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
			body: new URLSearchParams({ content: fileContent.value }),
		})

		const data = (await response.json()) as MclogsResponse

		if (data.success && data.url) {
			await navigator.clipboard.writeText(data.url)
			addNotification({
				title: 'Log URL copied',
				text: 'Your log file URL has been copied to your clipboard.',
				type: 'success',
			})
		} else {
			throw new Error(data.error)
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

function close() {
	resetState()
	emit('close')
}

onMounted(async () => {
	if (modulesLoaded) {
		await modulesLoaded
	}
})

onUnmounted(() => {
	editorInstance.value = null
	resetState()
})

defineExpose({
	saveFileContent,
	saveAndRestart,
	shareToMclogs,
	close,
	isEditingImage,
	fileContent,
})
</script>
