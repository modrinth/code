<template>
	<div
		class="relative flex h-[750px] items-center justify-center overflow-hidden rounded-[20px] bg-black"
	>
		<div v-if="state.hasError" class="flex flex-col items-center justify-center gap-4">
			<TriangleAlertIcon class="size-8 text-red" />
			<p class="m-0 text-secondary">
				{{ state.errorMessage || formatMessage(messages.invalidImage) }}
			</p>
		</div>
		<img
			v-show="isReady"
			ref="imageRef"
			:src="imageObjectUrl"
			class="max-h-full max-w-full rounded-lg object-contain"
			:class="{ 'cursor-zoom-in': !zoomed, 'cursor-zoom-out': zoomed }"
			:alt="formatMessage(messages.viewedImageAlt)"
			@load="handleImageLoad"
			@error="handleImageError"
			@click="toggleZoom"
		/>

		<div
			v-if="isReady"
			class="absolute bottom-4 left-1/2 flex -translate-x-1/2 items-center gap-1 rounded-2xl bg-surface-3/80 p-1.5 backdrop-blur-sm"
		>
			<ButtonStyled type="transparent">
				<button v-tooltip="formatMessage(messages.zoomIn)" @click="zoomIn">
					<ZoomInIcon />
				</button>
			</ButtonStyled>
			<ButtonStyled type="transparent">
				<button v-tooltip="formatMessage(messages.zoomOut)" @click="zoomOut">
					<ZoomOutIcon />
				</button>
			</ButtonStyled>
			<div class="mx-1 h-6 w-px bg-surface-5" />
			<ButtonStyled type="transparent">
				<button v-tooltip="formatMessage(messages.resetZoom)" @click="resetZoom">
					<span class="px-1 text-sm tabular-nums">{{ Math.round(scale * 100) }}%</span>
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { TriangleAlertIcon, ZoomInIcon, ZoomOutIcon } from '@modrinth/assets'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	invalidImage: {
		id: 'files.image_viewer.invalid_image',
		defaultMessage: 'Invalid or empty image file.',
	},
	viewedImageAlt: {
		id: 'files.image_viewer.viewed_image_alt',
		defaultMessage: 'Viewed image',
	},
	zoomIn: {
		id: 'files.image_viewer.zoom_in',
		defaultMessage: 'Zoom in',
	},
	zoomOut: {
		id: 'files.image_viewer.zoom_out',
		defaultMessage: 'Zoom out',
	},
	resetZoom: {
		id: 'files.image_viewer.reset_zoom',
		defaultMessage: 'Reset zoom',
	},
	imageTooLarge: {
		id: 'files.image_viewer.image_too_large',
		defaultMessage: 'Image too large to view (max {maxDimension}x{maxDimension} pixels)',
	},
	loadFailed: {
		id: 'files.image_viewer.load_failed',
		defaultMessage: 'Failed to load image',
	},
})

const MAX_IMAGE_DIMENSION = 4096

const props = defineProps<{
	imageBlob: Blob
}>()

const state = ref({
	isLoading: true,
	hasError: false,
	errorMessage: '',
})

const imageRef = ref<HTMLImageElement | null>(null)
const imageObjectUrl = ref('')
const scale = ref(1)
const zoomed = ref(false)

const isReady = computed(() => !state.value.isLoading && !state.value.hasError)

function updateImageUrl(blob: Blob) {
	if (imageObjectUrl.value) URL.revokeObjectURL(imageObjectUrl.value)
	imageObjectUrl.value = URL.createObjectURL(blob)
}

function handleImageLoad() {
	const img = imageRef.value
	if (img && (img.naturalWidth > MAX_IMAGE_DIMENSION || img.naturalHeight > MAX_IMAGE_DIMENSION)) {
		state.value.hasError = true
		state.value.errorMessage = formatMessage(messages.imageTooLarge, {
			maxDimension: MAX_IMAGE_DIMENSION,
		})
	}
	state.value.isLoading = false
}

function handleImageError() {
	state.value.isLoading = false
	state.value.hasError = true
	state.value.errorMessage = formatMessage(messages.loadFailed)
}

function toggleZoom() {
	if (zoomed.value) {
		resetZoom()
	} else {
		scale.value = 2
		zoomed.value = true
	}
}

function zoomIn() {
	scale.value = Math.min(scale.value * 1.25, 5)
	zoomed.value = scale.value > 1
}

function zoomOut() {
	scale.value = Math.max(scale.value * 0.8, 0.1)
	zoomed.value = scale.value > 1
}

function resetZoom() {
	scale.value = 1
	zoomed.value = false
}

watch(scale, (s) => {
	if (imageRef.value) {
		imageRef.value.style.transform = s === 1 ? '' : `scale(${s})`
		imageRef.value.style.transition = 'transform 0.2s ease-out'
	}
})

watch(
	() => props.imageBlob,
	(newBlob) => {
		if (!newBlob) return
		state.value.isLoading = true
		state.value.hasError = false
		scale.value = 1
		zoomed.value = false
		updateImageUrl(newBlob)
	},
)

onMounted(() => {
	if (props.imageBlob) updateImageUrl(props.imageBlob)
})

onUnmounted(() => {
	if (imageObjectUrl.value) URL.revokeObjectURL(imageObjectUrl.value)
})
</script>
