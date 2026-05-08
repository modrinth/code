<template>
	<div class="analytics-loading-bar" :style="{ opacity: isVisible ? 1 : 0 }" aria-hidden="true">
		<div
			class="analytics-loading-bar__track"
			:style="{
				width: `${progress}%`,
				transition: !isTransitioning
					? 'none'
					: isFinishing
						? 'width 0.1s ease-in-out'
						: isCreeping
							? 'width 2s linear'
							: 'width 0.9s ease-in-out',
			}"
		/>
	</div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'

const props = defineProps<{
	loading: boolean
}>()

const progress = ref(0)
const isVisible = ref(false)
const isFinishing = ref(false)
const isCreeping = ref(false)
const isTransitioning = ref(false)

let startFrame: number | null = null
let showFrame: number | null = null
let creepTimeout: ReturnType<typeof setTimeout> | null = null
let hideTimeout: ReturnType<typeof setTimeout> | null = null
let resetTimeout: ReturnType<typeof setTimeout> | null = null

function clearTimers() {
	if (showFrame !== null && typeof window !== 'undefined') {
		window.cancelAnimationFrame(showFrame)
	}
	if (startFrame !== null && typeof window !== 'undefined') {
		window.cancelAnimationFrame(startFrame)
	}
	if (creepTimeout) clearTimeout(creepTimeout)
	if (hideTimeout) clearTimeout(hideTimeout)
	if (resetTimeout) clearTimeout(resetTimeout)
	showFrame = null
	startFrame = null
	creepTimeout = null
	hideTimeout = null
	resetTimeout = null
}

function start() {
	clearTimers()
	isVisible.value = false
	progress.value = 0
	isFinishing.value = false
	isCreeping.value = false
	isTransitioning.value = false

	if (typeof window === 'undefined') {
		progress.value = 98
		return
	}

	showFrame = window.requestAnimationFrame(() => {
		isVisible.value = true
		showFrame = null
		startFrame = window.requestAnimationFrame(() => {
			isTransitioning.value = true
			progress.value = 85
			startFrame = null
		})
	})
	creepTimeout = setTimeout(() => {
		isCreeping.value = true
		progress.value = 98
		creepTimeout = null
	}, 900)
}

function finish() {
	clearTimers()
	isVisible.value = true
	isFinishing.value = true
	isCreeping.value = false
	isTransitioning.value = true
	progress.value = 100

	if (typeof window === 'undefined') {
		isVisible.value = false
		progress.value = 0
		isFinishing.value = false
		isCreeping.value = false
		isTransitioning.value = false
		return
	}

	hideTimeout = setTimeout(() => {
		isVisible.value = false
		resetTimeout = setTimeout(() => {
			isTransitioning.value = false
			progress.value = 0
			isFinishing.value = false
			isCreeping.value = false
		}, 400)
	}, 350)
}

watch(
	() => props.loading,
	(loading) => {
		if (loading) {
			start()
		} else if (
			isVisible.value ||
			progress.value > 0 ||
			showFrame !== null ||
			startFrame !== null ||
			creepTimeout !== null
		) {
			finish()
		}
	},
	{ immediate: true },
)

onBeforeUnmount(clearTimers)
</script>

<style scoped>
.analytics-loading-bar {
	position: absolute;
	top: 0;
	left: 0;
	right: 0;
	z-index: 20;
	height: 2px;
	overflow: hidden;
	background: color-mix(in srgb, var(--color-brand) 18%, transparent);
	pointer-events: none;
	transition: opacity 0.4s;
}

.analytics-loading-bar__track {
	height: 100%;
	border-radius: 999px;
	background: var(--loading-bar-gradient);
}
</style>
