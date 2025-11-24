<template>
	<component :is="icon" :class="{ animating: isAnimating }" ref="iconRef" />
</template>

<script setup>
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'

const props = defineProps({
	icon: { type: Object, required: true },
})

const isAnimating = ref(false)
const iconRef = ref(null)
let target = null
let AnimationLength = 0
let hover = false
let isInfinite = false

onMounted(async () => {
	await nextTick()
	const el = iconRef.value?.$el || iconRef.value
	if (!el) {
		console.warn('AnimatedIcon: Unable to find DOM element.')
		return
	}

	// calculate animation duration
	el.classList.add('animating')
	el.offsetHeight
	AnimationLength = calculateLongestAnimation(el)
	el.classList.remove('animating')

	// use parent if it exists, otherwise fallback to component itself
	target = el.parentElement || el
	target.addEventListener('mouseenter', playAnimation)
	target.addEventListener('mouseleave', stopHover)
})

onBeforeUnmount(() => {
	if (!target) return
	target.removeEventListener('mouseenter', playAnimation)
	target.removeEventListener('mouseleave', stopHover)
})

const playAnimation = () => {
	hover = true
	if (isAnimating.value) return

	isAnimating.value = true

	setTimeout(() => {
		isAnimating.value = false
		if (isInfinite && hover) {
			playAnimation()
		}
	}, AnimationLength)
}

function stopHover() {
	hover = false
}

function calculateLongestAnimation(el) {
	if (!el) return 0
	let maxDuration = 0

	const iconElements = [el, ...el.querySelectorAll('*')]
	iconElements.forEach((child) => {
		const style = getComputedStyle(child)
		const durations = style.animationDuration.split(',').map((s) => parseTimeMs(s))
		const delays = style.animationDelay.split(',').map((s) => parseTimeMs(s))
		const iterations = style.animationIterationCount.split(',').map((s) => {
			if (s === 'infinite') {
				isInfinite = true
				return 1
			}
			return parseFloat(s) || 1
		})

		durations.forEach((duration, i) => {
			const delay = delays[i] || 0
			const iter = iterations[i] || 1
			const total = duration * iter + delay
			if (total > maxDuration) maxDuration = total
		})
	})

	return maxDuration
}

function parseTimeMs(s) {
	const num = parseFloat(s)
	if (!num) return 0
	return num * (s.endsWith('ms') ? 1 : 1000)
}
</script>
