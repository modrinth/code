<template>
	<Transition
		enter-active-class="transition-all duration-300 ease-out overflow-hidden"
		enter-from-class="opacity-0 max-h-0"
		enter-to-class="opacity-100 max-h-40"
		leave-active-class="transition-all duration-200 ease-in overflow-hidden"
		leave-from-class="opacity-100 max-h-40"
		leave-to-class="opacity-0 max-h-0"
	>
		<Admonition v-if="ctx.uploadState?.value?.isUploading" type="info" class="mb-4">
			<template #icon>
				<UploadIcon class="h-6 w-6 flex-none text-brand-blue" />
			</template>
			<template #header>
				Uploading files ({{ ctx.uploadState.value.completedFiles }}/{{
					ctx.uploadState.value.totalFiles
				}})
				<span v-if="ctx.uploadState.value.currentFileName" class="font-normal text-secondary">
					— {{ ctx.uploadState.value.currentFileName }}
				</span>
			</template>
			<span class="text-secondary">
				{{ formatBytes(ctx.uploadState.value.uploadedBytes) }}
				/ {{ formatBytes(ctx.uploadState.value.totalBytes) }} ({{
					Math.round(uploadOverallProgress * 100)
				}}%)
			</span>
			<template v-if="ctx.cancelUpload" #top-right-actions>
				<ButtonStyled type="outlined" color="blue">
					<button class="!border" @click="ctx.cancelUpload?.()">Cancel</button>
				</ButtonStyled>
			</template>
			<template #progress>
				<ProgressBar :progress="uploadOverallProgress" :max="1" color="blue" full-width />
			</template>
		</Admonition>
	</Transition>
	<TransitionGroup
		name="fs-op"
		enter-active-class="transition-all duration-300 ease-out overflow-hidden"
		enter-from-class="opacity-0 max-h-0"
		enter-to-class="opacity-100 max-h-40"
		leave-active-class="transition-all duration-200 ease-in overflow-hidden"
		leave-from-class="opacity-100 max-h-40"
		leave-to-class="opacity-0 max-h-0"
	>
		<Admonition
			v-for="op in activeOperations"
			:key="`fs-op-${op.op}-${op.src}`"
			:type="op.state === 'done' ? 'success' : op.state?.startsWith('fail') ? 'critical' : 'info'"
			class="mb-4"
		>
			<template #icon="{ iconClass }">
				<PackageOpenIcon :class="iconClass" />
			</template>
			<template #header>
				Extracting {{ op.src.includes('https://') ? 'modpack from URL' : op.src }}
				<span v-if="op.state === 'done'" class="font-normal text-green"> — Done</span>
				<span v-else-if="op.state?.startsWith('fail')" class="font-normal text-red"> — Failed</span>
			</template>
			<span class="text-secondary">
				{{ 'bytes_processed' in op ? formatBytes(op.bytes_processed ?? 0) : '0 B' }} extracted
				<template v-if="'current_file' in op && op.current_file">
					— {{ op.current_file?.split('/')?.pop() }}
				</template>
			</span>
			<template v-if="op.id && ctx.dismissOperation" #top-right-actions>
				<ButtonStyled
					v-if="op.state !== 'done' && !op.state?.startsWith('fail')"
					type="outlined"
					color="blue"
				>
					<button class="!border" @click="ctx.dismissOperation?.(op.id!, 'cancel')">Cancel</button>
				</ButtonStyled>
				<ButtonStyled
					v-if="op.state === 'done' || op.state?.startsWith('fail')"
					circular
					type="transparent"
					hover-color-fill="background"
					:color="op.state === 'done' ? 'green' : 'red'"
				>
					<button @click="ctx.dismissOperation?.(op.id!, 'dismiss')">
						<XIcon />
					</button>
				</ButtonStyled>
			</template>
			<template #progress>
				<ProgressBar
					:progress="'progress' in op ? (op.progress ?? 0) : 0"
					:max="1"
					:color="op.state === 'done' ? 'green' : op.state?.startsWith('fail') ? 'red' : 'blue'"
					:waiting="op.state === 'queued' || !op.progress || op.progress === 0"
					full-width
				/>
			</template>
		</Admonition>
	</TransitionGroup>
</template>

<script setup lang="ts">
import { PackageOpenIcon, UploadIcon, XIcon } from '@modrinth/assets'
import { formatBytes } from '@modrinth/utils'
import { computed } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import ProgressBar from '#ui/components/base/ProgressBar.vue'

import { injectFileManager } from '../providers/file-manager'

const ctx = injectFileManager()

const activeOperations = computed(() => ctx.activeOperations?.value ?? [])

const uploadOverallProgress = computed(() => {
	const state = ctx.uploadState?.value
	if (!state || !state.isUploading || state.totalFiles === 0) return 0
	return Math.min((state.completedFiles + state.currentFileProgress) / state.totalFiles, 1)
})
</script>
