<template>
	<div>
		<Transition name="upload-status" @enter="onUploadStatusEnter" @leave="onUploadStatusLeave">
			<div v-show="isUploading" ref="uploadStatusRef" class="upload-status">
				<div
					ref="statusContentRef"
					v-bind="$attrs"
					:class="['flex flex-col p-4 text-sm text-contrast']"
				>
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-2 font-bold">
							<FolderOpenIcon class="size-4" />
							<span>
								<span class="capitalize">
									{{
										formatMessage(messages.fileUploads, {
											fileType: props.fileType ? props.fileType : formatMessage(messages.file),
										})
									}}
								</span>
								<span>{{
									activeUploads.length > 0
										? formatMessage(messages.uploadsLeft, { count: activeUploads.length })
										: ''
								}}</span>
							</span>
						</div>
					</div>

					<div class="mt-2 space-y-2">
						<div
							v-for="item in uploadQueue"
							:key="item.file.name"
							class="flex h-6 items-center justify-between gap-2 text-xs"
						>
							<div class="flex flex-1 items-center gap-2 truncate">
								<transition-group name="status-icon" mode="out-in">
									<SpinnerIcon
										v-show="item.status === 'uploading'"
										key="spinner"
										class="absolute !size-4 animate-spin"
									/>
									<CheckCircleIcon
										v-show="item.status === 'completed'"
										key="check"
										class="absolute size-4 text-green"
									/>
									<XCircleIcon
										v-show="
											item.status.includes('error') ||
											item.status === 'cancelled' ||
											item.status === 'incorrect-type'
										"
										key="error"
										class="absolute size-4 text-red"
									/>
								</transition-group>
								<span class="ml-6 truncate">{{ item.file.name }}</span>
								<span class="text-secondary">{{ item.size }}</span>
							</div>
							<div class="flex min-w-[80px] items-center justify-end gap-2">
								<template v-if="item.status === 'completed'">
									<span>{{ formatMessage(messages.done) }}</span>
								</template>
								<template v-else-if="item.status === 'error-file-exists'">
									<span class="text-red">{{ formatMessage(messages.failedFileExists) }}</span>
								</template>
								<template v-else-if="item.status === 'error-generic'">
									<span class="text-red">{{
										formatMessage(messages.failedGeneric, {
											error: item.error?.message || formatMessage(messages.unexpectedError),
										})
									}}</span>
								</template>
								<template v-else-if="item.status === 'incorrect-type'">
									<span class="text-red">{{ formatMessage(messages.failedIncorrectType) }}</span>
								</template>
								<template v-else>
									<template v-if="item.status === 'uploading'">
										<span>{{ item.progress }}%</span>
										<div class="h-1 w-20 overflow-hidden rounded-full bg-bg">
											<div
												class="h-full bg-contrast transition-all duration-200"
												:style="{ width: item.progress + '%' }"
											/>
										</div>
										<ButtonStyled color="red" type="transparent" @click="cancelUpload(item)">
											<button>{{ formatMessage(commonMessages.cancelButton) }}</button>
										</ButtonStyled>
									</template>
									<template v-else-if="item.status === 'cancelled'">
										<span class="text-red">{{ formatMessage(messages.cancelled) }}</span>
									</template>
									<template v-else>
										<span>{{ item.progress }}%</span>
										<div class="h-1 w-20 overflow-hidden rounded-full bg-bg">
											<div
												class="h-full bg-contrast transition-all duration-200"
												:style="{ width: item.progress + '%' }"
											/>
										</div>
									</template>
								</template>
							</div>
						</div>
					</div>
				</div>
			</div>
		</Transition>
	</div>
</template>

<script setup lang="ts">
import { CheckCircleIcon, FolderOpenIcon, SpinnerIcon, XCircleIcon } from '@modrinth/assets'
import { computed, nextTick, ref, watch } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient } from '#ui/providers/api-client'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { commonMessages } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()

const messages = defineMessages({
	file: {
		id: 'files.upload-dropdown.file',
		defaultMessage: 'File',
	},
	fileUploads: {
		id: 'files.upload-dropdown.file-uploads',
		defaultMessage: '{fileType} uploads',
	},
	uploadsLeft: {
		id: 'files.upload-dropdown.uploads-left',
		defaultMessage: ' - {count} left',
	},
	done: {
		id: 'files.upload-dropdown.done',
		defaultMessage: 'Done',
	},
	failedFileExists: {
		id: 'files.upload-dropdown.failed-file-exists',
		defaultMessage: 'Failed - File already exists',
	},
	failedGeneric: {
		id: 'files.upload-dropdown.failed-generic',
		defaultMessage: 'Failed - {error}',
	},
	unexpectedError: {
		id: 'files.upload-dropdown.unexpected-error',
		defaultMessage: 'An unexpected error occurred.',
	},
	failedIncorrectType: {
		id: 'files.upload-dropdown.failed-incorrect-type',
		defaultMessage: 'Failed - Incorrect file type',
	},
	cancelled: {
		id: 'files.upload-dropdown.cancelled',
		defaultMessage: 'Cancelled',
	},
	incorrectFileType: {
		id: 'files.upload-dropdown.incorrect-file-type',
		defaultMessage: 'Upload had incorrect file type',
	},
	uploadFailed: {
		id: 'files.upload-dropdown.upload-failed',
		defaultMessage: 'Upload failed',
	},
	failedToUpload: {
		id: 'files.upload-dropdown.failed-to-upload',
		defaultMessage: 'Failed to upload {fileName}',
	},
})

interface UploadItem {
	file: File
	progress: number
	status:
		| 'pending'
		| 'uploading'
		| 'completed'
		| 'error-file-exists'
		| 'error-generic'
		| 'cancelled'
		| 'incorrect-type'
	size: string
	uploader?: ReturnType<typeof client.kyros.files_v0.uploadFile>
	error?: Error
}

interface Props {
	currentPath: string
	fileType?: string
	marginBottom?: number
	acceptedTypes?: Array<string>
}

defineOptions({
	inheritAttrs: false,
})

const props = defineProps<Props>()

const emit = defineEmits<{
	uploadComplete: []
}>()

const uploadStatusRef = ref<HTMLElement | null>(null)
const statusContentRef = ref<HTMLElement | null>(null)
const uploadQueue = ref<UploadItem[]>([])

const isUploading = computed(() => uploadQueue.value.length > 0)
const activeUploads = computed(() =>
	uploadQueue.value.filter((item) => item.status === 'pending' || item.status === 'uploading'),
)

const onUploadStatusEnter = (el: Element) => {
	const height = (el as HTMLElement).scrollHeight + (props.marginBottom || 0)
	;(el as HTMLElement).style.height = '0'

	void (el as HTMLElement).offsetHeight
	;(el as HTMLElement).style.height = `${height}px`
}

const onUploadStatusLeave = (el: Element) => {
	const height = (el as HTMLElement).scrollHeight + (props.marginBottom || 0)
	;(el as HTMLElement).style.height = `${height}px`

	void (el as HTMLElement).offsetHeight
	;(el as HTMLElement).style.height = '0'
}

watch(
	uploadQueue,
	() => {
		if (!uploadStatusRef.value) return
		const el = uploadStatusRef.value
		const itemsHeight = uploadQueue.value.length * 32
		const headerHeight = 12
		const gap = 8
		const padding = 32
		const totalHeight = padding + headerHeight + gap + itemsHeight + (props.marginBottom || 0)
		el.style.height = `${totalHeight}px`
	},
	{ deep: true },
)

const formatFileSize = (bytes: number): string => {
	if (bytes < 1024) return bytes + ' B'
	if (bytes < 1024 ** 2) return (bytes / 1024).toFixed(1) + ' KB'
	if (bytes < 1024 ** 3) return (bytes / 1024 ** 2).toFixed(1) + ' MB'
	return (bytes / 1024 ** 3).toFixed(1) + ' GB'
}

const cancelUpload = (item: UploadItem) => {
	if (item.uploader && item.status === 'uploading') {
		item.uploader.cancel()
		item.status = 'cancelled'

		setTimeout(async () => {
			const index = uploadQueue.value.findIndex((qItem) => qItem.file.name === item.file.name)
			if (index !== -1) {
				uploadQueue.value.splice(index, 1)
				await nextTick()
			}
		}, 5000)
	}
}

const badFileTypeMsg = formatMessage(messages.incorrectFileType)

const uploadFile = async (file: File) => {
	const uploadItem: UploadItem = {
		file,
		progress: 0,
		status: 'pending',
		size: formatFileSize(file.size),
	}

	uploadQueue.value.push(uploadItem)

	try {
		if (
			props.acceptedTypes &&
			!props.acceptedTypes.includes(file.type) &&
			!props.acceptedTypes.some((type) => file.name.endsWith(type))
		) {
			throw new Error(badFileTypeMsg)
		}

		uploadItem.status = 'uploading'
		const filePath = `${props.currentPath}/${file.name}`.replace('//', '/')

		const uploader = client.kyros.files_v0.uploadFile(filePath, file, {
			onProgress: ({ progress }) => {
				const index = uploadQueue.value.findIndex((item) => item.file.name === file.name)
				if (index !== -1) {
					uploadQueue.value[index].progress = Math.round(progress)
				}
			},
		})
		uploadItem.uploader = uploader

		await uploader.promise
		const index = uploadQueue.value.findIndex((item) => item.file.name === file.name)
		if (index !== -1 && uploadQueue.value[index].status !== 'cancelled') {
			uploadQueue.value[index].status = 'completed'
			uploadQueue.value[index].progress = 100
		}

		await nextTick()

		setTimeout(async () => {
			const removeIndex = uploadQueue.value.findIndex((item) => item.file.name === file.name)
			if (removeIndex !== -1) {
				uploadQueue.value.splice(removeIndex, 1)
				await nextTick()
			}
		}, 5000)

		emit('uploadComplete')
	} catch (error) {
		console.error('Error uploading file:', error)
		const index = uploadQueue.value.findIndex((item) => item.file.name === file.name)
		if (index !== -1 && uploadQueue.value[index].status !== 'cancelled') {
			const target = uploadQueue.value[index]

			if (error instanceof Error) {
				if (error.message === badFileTypeMsg) {
					target.status = 'incorrect-type'
				} else if (target.progress === 100 && error.message.includes('401')) {
					target.status = 'error-file-exists'
				} else {
					target.status = 'error-generic'
					target.error = error
				}
			}
		}

		setTimeout(async () => {
			const removeIndex = uploadQueue.value.findIndex((item) => item.file.name === file.name)
			if (removeIndex !== -1) {
				uploadQueue.value.splice(removeIndex, 1)
				await nextTick()
			}
		}, 5000)

		if (error instanceof Error && error.message !== 'Upload cancelled') {
			addNotification({
				title: formatMessage(messages.uploadFailed),
				text: formatMessage(messages.failedToUpload, { fileName: file.name }),
				type: 'error',
			})
		}
	}
}

defineExpose({
	uploadFile,
	cancelUpload,
})
</script>

<style scoped>
.upload-status {
	overflow: hidden;
	transition: height 0.2s ease;
}

.upload-status-enter-active,
.upload-status-leave-active {
	transition: height 0.2s ease;
	overflow: hidden;
}

.upload-status-enter-from,
.upload-status-leave-to {
	height: 0 !important;
}

.status-icon-enter-active,
.status-icon-leave-active {
	transition: all 0.25s ease;
}

.status-icon-enter-from,
.status-icon-leave-to {
	transform: scale(0);
	opacity: 0;
}

.status-icon-enter-to,
.status-icon-leave-from {
	transform: scale(1);
	opacity: 1;
}
</style>
