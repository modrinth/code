<template>
	<div class="flex h-full w-full flex-col gap-4">
		<FilesRenameItemModal ref="renameModal" :item="file" @rename="handleRenameItem" />

		<FilesEditingNavbar
			:file-name="file?.name"
			:is-image="isEditingImage"
			:file-path="file?.path"
			class="-mt-2"
			:breadcrumb-segments="breadcrumbSegments"
			@cancel="handleCancel"
			@save="() => saveFileContent(true)"
			@save-as="saveFileContentAs"
			@save-restart="saveFileContentRestart"
			@share="requestShareLink"
			@navigate="(index) => emit('navigate', index)"
		/>

		<div class="flex flex-col shadow-md">
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
				<FilesImageViewer v-else-if="isEditingImage && imagePreview" :image-blob="imagePreview" />
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
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import FilesEditingNavbar from '~/components/ui/servers/FilesEditingNavbar.vue'
import FilesImageViewer from '~/components/ui/servers/FilesImageViewer.vue'
import FilesRenameItemModal from '~/components/ui/servers/FilesRenameItemModal.vue'

const props = defineProps<{
	file: { name: string; type: string; path: string } | null
	breadcrumbSegments: string[]
	editorComponent: any
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

	editor.commands.addCommand({
		name: 'save',
		bindKey: { win: 'Ctrl-S', mac: 'Command-S' },
		exec: () => saveFileContent(false),
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
