<template>
	<Admonition type="info">
		<template #icon>
			<UploadIcon class="h-6 w-6 flex-none text-brand-blue" />
		</template>
		<template #header>
			Uploading files ({{ state.completedFiles }}/{{ state.totalFiles }})
			<span v-if="state.currentFileName" class="font-normal text-secondary">
				— {{ state.currentFileName }}
			</span>
		</template>
		<span class="text-secondary">
			{{ formatBytes(state.uploadedBytes) }} / {{ formatBytes(state.totalBytes) }} ({{
				Math.round(overallProgress * 100)
			}}%)
		</span>
		<template v-if="cancelUpload" #top-right-actions>
			<ButtonStyled type="outlined" color="blue">
				<button class="!border" type="button" @click="cancelUpload()">Cancel</button>
			</ButtonStyled>
		</template>
		<template #progress>
			<ProgressBar :progress="overallProgress" :max="1" color="blue" full-width />
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { UploadIcon } from '@modrinth/assets'
import { formatBytes } from '@modrinth/utils'
import { computed } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import ProgressBar from '#ui/components/base/ProgressBar.vue'
import { injectModrinthServerContext } from '#ui/providers'

const ctx = injectModrinthServerContext()

const state = computed(() => ctx.uploadState.value)
const cancelUpload = computed(() => ctx.cancelUpload.value)

const overallProgress = computed(() => {
	const s = state.value
	if (!s.isUploading || s.totalFiles === 0) return 0
	return Math.min((s.completedFiles + s.currentFileProgress) / s.totalFiles, 1)
})
</script>
