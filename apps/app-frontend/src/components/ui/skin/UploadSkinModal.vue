<template>
	<ModalWrapper ref="modal" @on-hide="hide(true)">
		<template #title>
			<span class="text-lg font-extrabold text-contrast"> Upload skin texture </span>
		</template>
		<div class="relative">
			<div
				class="border-2 border-dashed border-highlight-gray rounded-xl h-[173px] flex flex-col items-center justify-center p-8 cursor-pointer bg-button-bg hover:bg-button-hover transition-colors relative"
				@click="triggerFileInput"
			>
				<p class="mx-auto mb-0 text-primary font-bold text-lg text-center flex items-center gap-2">
					<UploadIcon /> Select skin texture file
				</p>
				<p class="mx-auto mt-0 text-secondary text-sm text-center">
					Drag and drop or click here to browse
				</p>
				<input
					ref="fileInput"
					type="file"
					accept="image/png"
					class="hidden"
					@change="handleInputFileChange"
				/>
			</div>
		</div>
	</ModalWrapper>
</template>

<script setup lang="ts">
import { UploadIcon } from '@modrinth/assets'
import { injectNotificationManager } from '@modrinth/ui'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { onBeforeUnmount, ref, watch } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { get_dragged_skin_data } from '@/helpers/skins'

const { addNotification } = injectNotificationManager()

const modal = ref()
const fileInput = ref<HTMLInputElement>()
const unlisten = ref<() => void>()
const modalVisible = ref(false)

const emit = defineEmits<{
	(e: 'uploaded', data: ArrayBuffer): void
	(e: 'canceled'): void
}>()

function show(e?: MouseEvent) {
	modal.value?.show(e)
	modalVisible.value = true
	setupDragDropListener()
}

function hide(emitCanceled = false) {
	modal.value?.hide()
	modalVisible.value = false
	cleanupDragDropListener()
	resetState()
	if (emitCanceled) {
		emit('canceled')
	}
}

function resetState() {
	if (fileInput.value) fileInput.value.value = ''
}

function triggerFileInput() {
	fileInput.value?.click()
}

async function handleInputFileChange(e: Event) {
	const files = (e.target as HTMLInputElement).files
	if (!files || files.length === 0) {
		return
	}
	const file = files[0]
	const buffer = await file.arrayBuffer()
	await processData(buffer)
}

async function setupDragDropListener() {
	try {
		if (modalVisible.value) {
			await cleanupDragDropListener()
			unlisten.value = await getCurrentWebview().onDragDropEvent(async (event) => {
				if (event.payload.type !== 'drop') {
					return
				}

				if (!event.payload.paths || event.payload.paths.length === 0) {
					return
				}

				const filePath = event.payload.paths[0]

				try {
					const data = await get_dragged_skin_data(filePath)
					await processData(data.buffer)
				} catch (error) {
					addNotification({
						title: 'Error processing file',
						text: error instanceof Error ? error.message : 'Failed to read the dropped file.',
						type: 'error',
					})
				}
			})
		}
	} catch (error) {
		console.error('Failed to set up drag and drop listener:', error)
	}
}

async function cleanupDragDropListener() {
	if (unlisten.value) {
		unlisten.value()
		unlisten.value = undefined
	}
}

async function processData(buffer: ArrayBuffer) {
	emit('uploaded', buffer)
	hide()
}

watch(modalVisible, (isVisible) => {
	if (isVisible) {
		setupDragDropListener()
	} else {
		cleanupDragDropListener()
	}
})

onBeforeUnmount(() => {
	cleanupDragDropListener()
})

defineExpose({ show, hide })
</script>
