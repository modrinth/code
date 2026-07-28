<template>
	<div
		ref="outerRef"
		data-tauri-drag-region
		class="min-w-0 overflow-hidden pl-3"
		:class="{ 'breadcrumb-fade-mask': isOverflowing }"
		:style="isOverflowing ? { '--scroll-distance': `-${overflowAmount}px` } : undefined"
		@mouseenter="onMouseEnter"
		@mouseleave="onMouseLeave"
	>
		<div
			ref="innerRef"
			data-tauri-drag-region
			class="flex w-fit items-center gap-1"
			:class="{ 'breadcrumbs-scroll': isAnimating }"
			@animationiteration="onAnimationIteration"
		>
			<template v-for="(breadcrumb, index) in breadcrumbs" :key="breadcrumb.slot">
				<router-link
					v-if="index < breadcrumbs.length - 1 && breadcrumb.to"
					:to="breadcrumb.to"
					class="shrink-0 whitespace-nowrap text-primary"
				>
					{{ breadcrumb.label }}
				</router-link>
				<span
					v-else
					data-tauri-drag-region
					class="shrink-0 whitespace-nowrap text-contrast font-semibold cursor-default select-none"
				>
					{{ breadcrumb.label }}
				</span>
				<ChevronRightIcon
					v-if="index < breadcrumbs.length - 1"
					data-tauri-drag-region
					class="w-5 h-5 shrink-0"
				/>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronRightIcon } from '@modrinth/assets'
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { injectBreadcrumbManager } from '@/providers/breadcrumbs'

const { entries: breadcrumbs } = injectBreadcrumbManager()

// Overflow detection
const outerRef = ref<HTMLDivElement | null>(null)
const innerRef = ref<HTMLDivElement | null>(null)
const isOverflowing = ref(false)
const isAnimating = ref(false)
const overflowAmount = ref(0)

let hovered = false
let stopping = false

function checkOverflow() {
	if (!outerRef.value || !innerRef.value) return
	const overflow = innerRef.value.scrollWidth - outerRef.value.clientWidth
	isOverflowing.value = overflow > 0
	overflowAmount.value = overflow + 12
}

function onMouseEnter() {
	hovered = true
	stopping = false
	if (isOverflowing.value) {
		isAnimating.value = true
	}
}

function onMouseLeave() {
	hovered = false
	if (isAnimating.value) {
		stopping = true
	}
}

function onAnimationIteration() {
	if (stopping && !hovered) {
		isAnimating.value = false
		stopping = false
	}
}

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
	checkOverflow()
	resizeObserver = new ResizeObserver(checkOverflow)
	if (outerRef.value) resizeObserver.observe(outerRef.value)
	if (innerRef.value) resizeObserver.observe(innerRef.value)
})

onBeforeUnmount(() => {
	resizeObserver?.disconnect()
})

watch(breadcrumbs, () => {
	requestAnimationFrame(checkOverflow)
})
</script>

<style scoped>
.breadcrumb-fade-mask {
	mask-image: linear-gradient(
		to right,
		transparent,
		black 12px,
		black calc(100% - 12px),
		transparent
	);
}

.breadcrumbs-scroll {
	animation: breadcrumb-scroll 10s ease-in-out infinite;
}

@keyframes breadcrumb-scroll {
	0% {
		transform: translateX(0);
	}
	35%,
	65% {
		transform: translateX(var(--scroll-distance));
	}
	100% {
		transform: translateX(0);
	}
}
</style>
