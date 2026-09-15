<template>
	<img
		ref="img"
		:src="src"
		v-bind="$attrs"
		@contextmenu="onContextMenu"
		@dragstart="onDragStart"
	/>
</template>

<script setup lang="ts">
import { useTemplateRef } from 'vue'

defineOptions({
	inheritAttrs: false,
})

const props = defineProps<{
	src?: string | null
	rawSrc?: string | null
}>()

const img = useTemplateRef<HTMLImageElement>('img')

defineExpose({
	img,
})

function swapImageSrc(
	imgEl: HTMLImageElement,
	fullUrl: string,
	revertTargets: { target: EventTarget; type: string; capture?: boolean }[],
) {
	const originalSrc = imgEl.src
	if (originalSrc === fullUrl) return

	imgEl.src = fullUrl

	let reverted = false
	const revert = () => {
		if (reverted) return
		reverted = true
		imgEl.src = originalSrc
		for (const { target, type, capture } of revertTargets) {
			target.removeEventListener(type, revert, capture)
		}
	}

	for (const { target, type, capture } of revertTargets) {
		target.addEventListener(type, revert, capture)
	}
}

function onContextMenu(event: MouseEvent) {
	if (!props.rawSrc) return

	swapImageSrc(event.currentTarget as HTMLImageElement, props.rawSrc, [
		{ target: window, type: 'focus' },
		{ target: window, type: 'pointerdown', capture: true },
		{ target: window, type: 'keydown', capture: true },
		{ target: window, type: 'pointermove', capture: true },
		{ target: window, type: 'wheel', capture: true },
		{ target: window, type: 'touchstart', capture: true },
	])
}

function onDragStart(event: DragEvent) {
	if (!props.rawSrc || !event.dataTransfer) return

	const filename = props.rawSrc.split('/').pop()?.split('?')[0] || 'image'

	event.dataTransfer.setData('DownloadURL', `application/octet-stream:${filename}:${props.rawSrc}`)
	event.dataTransfer.setData('text/uri-list', props.rawSrc)
	event.dataTransfer.setData('text/plain', props.rawSrc)
}
</script>
