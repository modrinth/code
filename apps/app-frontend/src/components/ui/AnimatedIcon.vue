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
let longestAnimation = 0
let target = null

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
	longestAnimation = calculateLongestAnimation(el)
	el.classList.remove('animating')

	// use parent if it exists, otherwise fallback to component itself
	target = el.parentElement || el
	target.addEventListener('mouseenter', playAnimation)
})

onBeforeUnmount(() => {
	if (target) target.removeEventListener('mouseenter', playAnimation)
})

const playAnimation = () => {
	if (isAnimating.value) return

	isAnimating.value = true

	setTimeout(() => {
		isAnimating.value = false
	}, longestAnimation)
}

function calculateLongestAnimation(el) {
	if (!el) return 0
	let maxDuration = 0

	const iconElements = [el, ...el.querySelectorAll('*')]
	iconElements.forEach((child) => {
		const style = getComputedStyle(child)
		const durations = style.animationDuration.split(',').map((s) => parseFloat(s) || 0)
		const delays = style.animationDelay.split(',').map((s) => parseFloat(s) || 0)

		durations.forEach((d, i) => {
			const delay = delays[i] || 0
			const total = (d + delay) * 1000
			if (total > maxDuration) maxDuration = total
		})
	})

	return maxDuration
}
</script>
