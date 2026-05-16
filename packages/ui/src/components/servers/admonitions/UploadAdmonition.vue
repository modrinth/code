<template>
	<Admonition type="info" :progress="displayProgress" progress-color="blue">
		<template #icon>
			<UploadIcon class="h-6 w-6 flex-none text-brand-blue" />
		</template>
		<template #header>
			{{ headerText }}
		</template>
		<span class="text-secondary">
			{{ formatBytes(displayUploadedBytes) }} / {{ formatBytes(state.totalBytes) }} ({{
				Math.round(displayProgress * 100)
			}}%)
		</span>
		<template v-if="cancelUpload" #top-right-actions>
			<ButtonStyled type="outlined" color="blue">
				<button class="!border" type="button" @click="cancelUpload()">Cancel</button>
			</ButtonStyled>
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { UploadIcon } from '@modrinth/assets'
import { computed } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { useFormatBytes } from '#ui/composables'
import { injectModrinthServerContext } from '#ui/providers'

const formatBytes = useFormatBytes()

const ctx = injectModrinthServerContext()

const state = computed(() => ctx.uploadState.value)
const cancelUpload = computed(() => ctx.cancelUpload.value)

const headerText = computed(() => {
	const s = state.value
	if (s.currentFileName) {
		return `Uploading ${s.currentFileName} (${currentFileNumber.value}/${s.totalFiles})`
	}
	return `Uploading files (${s.completedFiles}/${s.totalFiles})`
})

const currentFileNumber = computed(() => {
	const s = state.value
	if (s.totalFiles === 0) return 0
	return Math.min(s.completedFiles + 1, s.totalFiles)
})

const displayUploadedBytes = computed(() => {
	const s = state.value
	if (s.totalBytes <= 0) return s.uploadedBytes
	return Math.min(s.uploadedBytes, s.totalBytes)
})

const displayProgress = computed(() => {
	const s = state.value
	if (!s.isUploading) return 0
	if (s.totalBytes > 0) {
		return displayUploadedBytes.value / s.totalBytes
	}
	return 0
})
</script>
