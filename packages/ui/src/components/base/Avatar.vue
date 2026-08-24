<template>
	<img
		v-if="src && !failed"
		ref="img"
		class="avatar shrink-0"
		:style="`--_size: ${cssSize}`"
		:class="{
			circle: circle,
			detecting: !hasDetectedCorners,
			'no-shadow': noShadow,
			padded: hasTransparentCorners && !circle && !disableConditionalIconPadding,
			raised: raised,
			pixelated: pixelated,
		}"
		:src="src"
		:alt="alt"
		:loading="loading"
		@load="onLoad"
		@error="onError"
	/>
	<svg
		v-else
		class="avatar shrink-0"
		:style="`--_size: ${cssSize}${tint ? `;--_tint:oklch(50% 75% ${tint})` : ''}`"
		:class="{
			tint: tint,
			circle: circle,
			'no-shadow': noShadow,
			raised: raised,
		}"
		xml:space="preserve"
		fill-rule="evenodd"
		stroke-linecap="round"
		stroke-linejoin="round"
		stroke-miterlimit="1.5"
		clip-rule="evenodd"
		viewBox="0 0 104 104"
		aria-hidden="true"
	>
		<path fill="none" d="M0 0h103.4v103.4H0z" />
		<path
			fill="none"
			stroke="#9a9a9a"
			stroke-width="5"
			d="M51.7 92.5V51.7L16.4 31.3l35.3 20.4L87 31.3 51.7 11 16.4 31.3v40.8l35.3 20.4L87 72V31.3L51.7 11"
		/>
	</svg>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'

const pixelated = ref(false)
const hasTransparentCorners = ref(false)
const hasDetectedCorners = ref(false)
const img = useTemplateRef<HTMLImageElement>('img')
const failed = ref(false)
let detectionTimeout: number | undefined
let detectingSource: string | undefined

const DETECTION_TIMEOUT_MS = 2000
const CORNER_SAMPLE_SIZE = 8

defineExpose({
	failed,
})

const props = withDefaults(
	defineProps<{
		src?: string | null
		alt?: string
		size?: string
		circle?: boolean
		noShadow?: boolean
		disableConditionalIconPadding?: boolean
		loading?: 'eager' | 'lazy'
		raised?: boolean
		tintBy?: string | null
	}>(),
	{
		src: null,
		alt: '',
		size: '2rem',
		circle: false,
		noShadow: false,
		disableConditionalIconPadding: false,
		loading: 'eager',
		raised: false,
		tintBy: null,
	},
)

const LEGACY_PRESETS: Record<string, string> = {
	xxs: '1.25rem',
	xs: '2.5rem',
	sm: '3rem',
	md: '6rem',
	lg: '9rem',
}

const cssSize = computed(() => LEGACY_PRESETS[props.size] ?? props.size)

watch(
	() => props.src,
	() => {
		clearDetectionTimeout()
		detectingSource = undefined
		failed.value = false
		hasTransparentCorners.value = false
		hasDetectedCorners.value = false
	},
)

onMounted(() => {
	const image = img.value
	if (!image?.complete || hasDetectedCorners.value) return

	if (image.naturalWidth > 0) {
		onLoad()
	} else {
		failed.value = true
	}
})

onBeforeUnmount(clearDetectionTimeout)

function clearDetectionTimeout() {
	if (detectionTimeout !== undefined) {
		window.clearTimeout(detectionTimeout)
		detectionTimeout = undefined
	}
}

function onError(e) {
	clearDetectionTimeout()
	detectingSource = undefined
	console.log('Avatar image failed to load:', props.src, e)
	failed.value = true
}

function onLoad() {
	const image = img.value
	if (!image) return
	const source = image.currentSrc
	if (detectingSource === source) return
	detectingSource = source
	clearDetectionTimeout()

	if (image.naturalWidth && image.naturalWidth < 32) {
		pixelated.value = true
	} else {
		pixelated.value = false
	}

	if (canReadImagePixels(source)) {
		const transparentCorners = detectTransparentCorners(image)
		if (transparentCorners !== null) {
			detectingSource = undefined
			hasTransparentCorners.value = transparentCorners
			hasDetectedCorners.value = true
			return
		}
	}

	const probe = new Image()
	let detectionFinished = false
	probe.crossOrigin = 'anonymous'
	probe.onload = () => {
		if (detectionFinished || img.value?.currentSrc !== source) return
		detectionFinished = true
		detectingSource = undefined
		clearDetectionTimeout()
		hasTransparentCorners.value = detectTransparentCorners(probe) ?? false
		hasDetectedCorners.value = true
	}
	probe.onerror = () => {
		if (detectionFinished || img.value?.currentSrc !== source) return
		detectionFinished = true
		detectingSource = undefined
		clearDetectionTimeout()
		hasDetectedCorners.value = true
	}
	detectionTimeout = window.setTimeout(() => {
		if (img.value?.currentSrc !== source) return
		detectionFinished = true
		detectingSource = undefined
		detectionTimeout = undefined
		hasDetectedCorners.value = true
	}, DETECTION_TIMEOUT_MS)
	probe.src = source
}

function canReadImagePixels(source: string): boolean {
	try {
		const url = new URL(source, window.location.href)
		return url.protocol === 'data:' || url.origin === window.location.origin
	} catch {
		return false
	}
}

function detectTransparentCorners(image: HTMLImageElement): boolean | null {
	if (image.naturalWidth < CORNER_SAMPLE_SIZE || image.naturalHeight < CORNER_SAMPLE_SIZE) {
		return false
	}

	const canvas = document.createElement('canvas')
	canvas.width = CORNER_SAMPLE_SIZE * 2
	canvas.height = CORNER_SAMPLE_SIZE * 2
	const context = canvas.getContext('2d', { willReadFrequently: true })
	if (!context) return false

	const right = image.naturalWidth - CORNER_SAMPLE_SIZE
	const bottom = image.naturalHeight - CORNER_SAMPLE_SIZE
	const drawCorner = (
		sourceX: number,
		sourceY: number,
		destinationX: number,
		destinationY: number,
	) =>
		context.drawImage(
			image,
			sourceX,
			sourceY,
			CORNER_SAMPLE_SIZE,
			CORNER_SAMPLE_SIZE,
			destinationX,
			destinationY,
			CORNER_SAMPLE_SIZE,
			CORNER_SAMPLE_SIZE,
		)

	try {
		drawCorner(0, 0, 0, 0)
		drawCorner(right, 0, CORNER_SAMPLE_SIZE, 0)
		drawCorner(0, bottom, 0, CORNER_SAMPLE_SIZE)
		drawCorner(right, bottom, CORNER_SAMPLE_SIZE, CORNER_SAMPLE_SIZE)
		const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data
		for (let alpha = 3; alpha < pixels.length; alpha += 4) {
			if (pixels[alpha] !== 0) return false
		}
		return true
	} catch {
		return null
	}
}

const tint = computed(() => {
	if (props.tintBy) {
		return hash(props.tintBy) % 360
	} else {
		return null
	}
})

function hash(str: string): number {
	let hash = 0
	for (let i = 0, len = str.length; i < len; i++) {
		const chr = str.charCodeAt(i)
		hash = (hash << 5) - hash + chr
		hash |= 0
	}
	return hash
}
</script>

<style lang="scss" scoped>
.avatar {
	--_size: 2rem;

	outline: 1px solid rgb(255 255 255 / 15%);
	outline-offset: -1px;
	background-color: var(--color-button-bg);
	object-fit: contain;
	border-radius: calc(16 / 96 * var(--_override-size, var(--_size)));
	position: relative;
	height: var(--_override-size, var(--_size));
	width: var(--_override-size, var(--_size));
	min-height: var(--_override-size, var(--_size));
	min-width: var(--_override-size, var(--_size));

	&.circle {
		border-radius: 50%;
	}

	&:not(.no-shadow) {
		box-shadow: var(--shadow-card);
	}

	&.no-shadow {
		box-shadow: none;
	}

	&.pixelated {
		image-rendering: pixelated;
	}

	&.detecting {
		visibility: hidden;
	}

	&.padded {
		padding: calc(6 / 96 * var(--_override-size, var(--_size)));
	}

	&.raised {
		background-color: var(--color-raised-bg);
	}

	&.tint {
		background-color: color-mix(in oklch, var(--color-button-bg) 100%, var(--_tint) 5%);
	}
}
</style>
