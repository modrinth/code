<script setup lang="ts">
import { useElementSize } from '@vueuse/core'
import { computed, ref } from 'vue'

const props = defineProps<{ path: string }>()
const label = ref<HTMLElement>()
const { width } = useElementSize(label)
let canvas: HTMLCanvasElement | undefined

const displayPath = computed(() => {
	const element = label.value
	if (!element || !width.value) return props.path
	canvas ??= element.ownerDocument.createElement('canvas')
	const context = canvas.getContext('2d')
	const style = element.ownerDocument.defaultView?.getComputedStyle(element)
	if (!context || !style) return props.path
	context.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`
	if (context.measureText(props.path).width <= width.value) return props.path

	const characters = Array.from(props.path)
	let start = 0
	let end = characters.length - 1
	let result = '…'
	while (start <= end) {
		const count = Math.floor((start + end) / 2)
		const prefixLength = Math.ceil(count / 3)
		const suffixLength = count - prefixLength
		const candidate = `${characters.slice(0, prefixLength).join('')}…${suffixLength ? characters.slice(-suffixLength).join('') : ''}`
		if (context.measureText(candidate).width <= width.value) {
			result = candidate
			start = count + 1
		} else {
			end = count - 1
		}
	}
	return result
})
</script>

<template>
	<span ref="label" class="block min-w-0 flex-1 overflow-hidden whitespace-nowrap" :title="path">
		{{ displayPath }}
	</span>
</template>
