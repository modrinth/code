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
				{{
					formatMessage(messages.uploadingFiles, {
						completed: ctx.uploadState.value.completedFiles,
						total: ctx.uploadState.value.totalFiles,
					})
				}}
				<span v-if="ctx.uploadState.value.currentFileName" class="font-normal text-secondary">
					— {{ ctx.uploadState.value.currentFileName }}
				</span>
			</template>
			<span class="text-secondary">
				{{
					formatMessage(messages.uploadProgress, {
						uploaded: formatBytes(ctx.uploadState.value.uploadedBytes),
						total: formatBytes(ctx.uploadState.value.totalBytes),
						percent: Math.round(uploadOverallProgress * 100),
					})
				}}
			</span>
			<template v-if="ctx.cancelUpload" #top-right-actions>
				<ButtonStyled type="outlined" color="blue">
					<button class="!border" @click="ctx.cancelUpload?.()">
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
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
				{{
					formatMessage(messages.extracting, {
						source: op.src.includes('https://') ? formatMessage(messages.modpackFromUrl) : op.src,
					})
				}}
				<span v-if="op.state === 'done'" class="font-normal text-green">
					— {{ formatMessage(messages.done) }}</span
				>
				<span v-else-if="op.state?.startsWith('fail')" class="font-normal text-red">
					— {{ formatMessage(messages.failed) }}</span
				>
			</template>
			<span class="text-secondary">
				{{
					formatMessage(messages.extracted, {
						size: 'bytes_processed' in op ? formatBytes(op.bytes_processed ?? 0) : '0 B',
					})
				}}
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
					<button class="!border" @click="ctx.dismissOperation?.(op.id!, 'cancel')">
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
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
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { injectFileManager } from '../providers/file-manager'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	uploadingFiles: {
		id: 'files.operations.uploading-files',
		defaultMessage: 'Uploading files ({completed}/{total})',
	},
	uploadProgress: {
		id: 'files.operations.upload-progress',
		defaultMessage: '{uploaded} / {total} ({percent}%)',
	},
	extracting: {
		id: 'files.operations.extracting',
		defaultMessage: 'Extracting {source}',
	},
	modpackFromUrl: {
		id: 'files.operations.modpack-from-url',
		defaultMessage: 'modpack from URL',
	},
	done: {
		id: 'files.operations.done',
		defaultMessage: 'Done',
	},
	failed: {
		id: 'files.operations.failed',
		defaultMessage: 'Failed',
	},
	extracted: {
		id: 'files.operations.extracted',
		defaultMessage: '{size} extracted',
	},
})

const ctx = injectFileManager()

const activeOperations = computed(() => ctx.activeOperations?.value ?? [])

const uploadOverallProgress = computed(() => {
	const state = ctx.uploadState?.value
	if (!state || !state.isUploading || state.totalFiles === 0) return 0
	return Math.min((state.completedFiles + state.currentFileProgress) / state.totalFiles, 1)
})
</script>
