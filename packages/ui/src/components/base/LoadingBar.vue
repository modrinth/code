<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { injectLoadingState } from '#ui/providers/loading-state'

const props = withDefaults(
	defineProps<{
		/** Bar height in pixels. */
		height?: number
		/** Background gradient. Defaults to the brand green. */
		color?: string
		/** Total bar fill duration in ms (visual progress easing). */
		duration?: number
		/** Delay in ms before the bar becomes visible after a load begins. */
		throttle?: number
		/** CSS position. Use `absolute` when wrapping in a custom positioned container (e.g. desktop top-bar offset). */
		position?: 'fixed' | 'absolute'
		/** Top offset CSS value. */
		offsetTop?: string
		/** Left offset CSS value. */
		offsetLeft?: string
		/** Right offset CSS value. */
		offsetRight?: string
	}>(),
	{
		height: 2,
		color: 'var(--loading-bar-gradient)',
		duration: 1000,
		throttle: 0,
		position: 'fixed',
		offsetTop: '0',
		offsetLeft: '0',
		offsetRight: '0',
	},
)

const loadingState = injectLoadingState(null)

const progress = ref(0)
const isVisible = ref(false)
const step = computed(() => 10000 / props.duration)

let _timer: ReturnType<typeof setInterval> | null = null
let _throttle: ReturnType<typeof setTimeout> | null = null
let _hideTimeout: ReturnType<typeof setTimeout> | null = null
let _resetTimeout: ReturnType<typeof setTimeout> | null = null

function clearTimers() {
	if (_timer) clearInterval(_timer)
	if (_throttle) clearTimeout(_throttle)
	if (_hideTimeout) clearTimeout(_hideTimeout)
	if (_resetTimeout) clearTimeout(_resetTimeout)
	_timer = null
	_throttle = null
	_hideTimeout = null
	_resetTimeout = null
}

function startTimer() {
	if (typeof window === 'undefined') return
	_timer = setInterval(() => {
		progress.value = Math.min(100, progress.value + step.value)
	}, 100)
}

function start() {
	clearTimers()
	progress.value = 0
	if (props.throttle && typeof window !== 'undefined') {
		_throttle = setTimeout(() => {
			isVisible.value = true
			startTimer()
		}, props.throttle)
	} else {
		isVisible.value = true
		startTimer()
	}
}

function finish() {
	progress.value = 100
	clearTimers()
	if (typeof window === 'undefined') {
		isVisible.value = false
		progress.value = 0
		return
	}
	_hideTimeout = setTimeout(() => {
		isVisible.value = false
		_resetTimeout = setTimeout(() => {
			progress.value = 0
		}, 400)
	}, 500)
}

if (loadingState) {
	watch(
		() => loadingState.pending.value && loadingState.barEnabled.value,
		(active) => {
			if (active) start()
			else finish()
		},
		{ immediate: true },
	)
}

onBeforeUnmount(clearTimers)
</script>

<template>
	<div
		class="modrinth-loading-bar"
		:style="{
			position: props.position,
			top: props.offsetTop,
			right: props.offsetRight,
			left: props.offsetLeft,
			pointerEvents: 'none',
			width: `${progress}%`,
			height: `${isVisible ? props.height : 0}px`,
			borderRadius: `${props.height}px`,
			background: props.color,
			backgroundSize: `${(100 / Math.max(progress, 0.01)) * 100}% auto`,
			opacity: isVisible ? 1 : 0,
			transition: 'width 0.1s ease-in-out, height 0.1s ease-out, opacity 0.4s',
		}"
	/>
</template>

<style lang="scss" scoped>
.modrinth-loading-bar {
	z-index: 999999;

	&::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		width: 100%;
		background-image: radial-gradient(80% 100% at 20% 0%, var(--color-brand) 0%, transparent 80%);
		opacity: 0.1;
		z-index: -1;
		pointer-events: none;
	}
}
</style>
