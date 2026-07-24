<template>
	<div
		ref="metadata"
		class="page-header-metadata flex min-w-0 flex-wrap items-center gap-x-[1.625rem] gap-y-2"
	>
		<slot />
	</div>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'

const metadata = ref<HTMLElement | null>(null)
let resizeObserver: ResizeObserver | null = null
let mutationObserver: MutationObserver | null = null
const observedItems = new Set<Element>()

function getItems() {
	return Array.from(
		metadata.value?.querySelectorAll<HTMLElement>('[data-page-header-metadata-item]') ?? [],
	)
}

function observeItems() {
	if (!resizeObserver) return

	for (const item of getItems()) {
		if (observedItems.has(item)) continue

		resizeObserver.observe(item)
		observedItems.add(item)
	}
}

function updateRowStarts() {
	const root = metadata.value
	if (!root) return

	const isRtl = getComputedStyle(root).direction === 'rtl'
	let previousOffset: number | null = null

	for (const item of getItems()) {
		const offset = item.offsetLeft
		const startsRow =
			previousOffset === null ||
			(isRtl ? offset >= previousOffset - 1 : offset <= previousOffset + 1)

		item.toggleAttribute('data-page-header-metadata-row-start', startsRow)
		previousOffset = offset
	}
}

function scheduleUpdate() {
	void nextTick(() => {
		observeItems()
		updateRowStarts()
	})
}

onMounted(() => {
	scheduleUpdate()

	if (typeof ResizeObserver !== 'undefined') {
		resizeObserver = new ResizeObserver(scheduleUpdate)
		if (metadata.value) {
			resizeObserver.observe(metadata.value)
		}
		observeItems()
	}

	if (typeof MutationObserver !== 'undefined' && metadata.value) {
		mutationObserver = new MutationObserver(scheduleUpdate)
		mutationObserver.observe(metadata.value, { childList: true, subtree: true })
	}
})

onBeforeUnmount(() => {
	resizeObserver?.disconnect()
	mutationObserver?.disconnect()
	observedItems.clear()
})
</script>

<style scoped>
.page-header-metadata
	:deep([data-page-header-metadata-item]:first-child .page-header-metadata-item-divider),
.page-header-metadata
	:deep([data-page-header-metadata-row-start] .page-header-metadata-item-divider) {
	display: none;
}
</style>
