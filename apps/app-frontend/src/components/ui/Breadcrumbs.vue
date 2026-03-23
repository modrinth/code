<template>
	<div
		ref="outerRef"
		data-tauri-drag-region
		class="min-w-0 overflow-hidden pl-3"
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
			{{ breadcrumbData.resetToNames(breadcrumbs) }}
			<template v-for="breadcrumb in breadcrumbs" :key="breadcrumb.name">
				<router-link
					v-if="breadcrumb.link"
					:to="{
						path: breadcrumb.link.replace('{id}', encodeURIComponent($route.params.id as string)),
						query: breadcrumb.query,
					}"
					class="shrink-0 whitespace-nowrap text-primary"
				>
					{{ resolveLabel(breadcrumb.name) }}
				</router-link>
				<span
					v-else
					data-tauri-drag-region
					class="shrink-0 whitespace-nowrap text-contrast font-semibold cursor-default select-none"
				>
					{{ resolveLabel(breadcrumb.name) }}
				</span>
				<ChevronRightIcon v-if="breadcrumb.link" data-tauri-drag-region class="w-5 h-5 shrink-0" />
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronRightIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import { useBreadcrumbs } from '@/store/breadcrumbs'

interface Breadcrumb {
	name: string
	link?: string
	query?: Record<string, string>
}

const route = useRoute()
const breadcrumbData = useBreadcrumbs()

const breadcrumbs = computed<Breadcrumb[]>(() => {
	const additionalContext =
		route.meta.useContext === true
			? breadcrumbData.context
			: route.meta.useRootContext === true
				? breadcrumbData.rootContext
				: null
	const crumbs = (route.meta.breadcrumb ?? []) as Breadcrumb[]
	return additionalContext ? [additionalContext as Breadcrumb, ...crumbs] : crumbs
})

function resolveLabel(name: string): string {
	return name.charAt(0) === '?' ? breadcrumbData.getName(name.slice(1)) : name
}

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
