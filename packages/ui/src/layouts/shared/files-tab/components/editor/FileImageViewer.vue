<template>
	<div class="relative flex h-[750px] items-center justify-center overflow-hidden rounded-[20px] bg-black">
		<div
			v-if="state.hasError"
			class="flex flex-col items-center justify-center gap-4"
		>
			<TriangleAlertIcon class="size-8 text-red" />
			<p class="m-0 text-secondary">{{ state.errorMessage || 'Invalid or empty image file.' }}</p>
		</div>
		<img
			v-show="isReady"
			ref="imageRef"
			:src="imageObjectUrl"
			class="max-h-full max-w-full rounded-lg object-contain"
			:class="{ 'cursor-zoom-in': !zoomed, 'cursor-zoom-out': zoomed }"
			alt="Viewed image"
			@load="handleImageLoad"
			@error="handleImageError"
			@click="toggleZoom"
		/>

		<div
			v-if="isReady"
			class="absolute bottom-4 left-1/2 flex -translate-x-1/2 items-center gap-1 rounded-2xl bg-surface-3/80 p-1.5 backdrop-blur-sm"
		>
			<ButtonStyled type="transparent">
				<button v-tooltip="'Zoom in'" @click="zoomIn">
					<ZoomInIcon />
				</button>
			</ButtonStyled>
			<ButtonStyled type="transparent">
				<button v-tooltip="'Zoom out'" @click="zoomOut">
					<ZoomOutIcon />
				</button>
			</ButtonStyled>
			<div class="mx-1 h-6 w-px bg-surface-5" />
			<ButtonStyled type="transparent">
				<button v-tooltip="'Reset zoom'" @click="resetZoom">
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
		state.value.errorMessage = `Image too large to view (max ${MAX_IMAGE_DIMENSION}x${MAX_IMAGE_DIMENSION} pixels)`
	}
	state.value.isLoading = false
}

function handleImageError() {
	state.value.isLoading = false
	state.value.hasError = true
	state.value.errorMessage = 'Failed to load image'
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
